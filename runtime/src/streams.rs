//! Stream API for asynchronous data processing
//!
//! This module provides a functional Stream API inspired by ReactiveX,
//! Project Reactor, and Kotlin Flows for processing asynchronous data streams.

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;

/// Represents a subscription to a stream that can be cancelled
#[derive(Debug, Clone)]
pub struct Subscription {
    cancelled: Arc<Mutex<bool>>,
}

impl Subscription {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(Mutex::new(false)),
        }
    }

    pub fn unsubscribe(&self) {
        *self.cancelled.lock().unwrap() = true;
    }

    pub fn is_subscribed(&self) -> bool {
        !*self.cancelled.lock().unwrap()
    }
}

impl Default for Subscription {
    fn default() -> Self {
        Self::new()
    }
}

/// Core Stream trait defining the functional API
pub trait Stream<T: Send>: Send {
    /// Transform each element using a mapper function
    fn map<U, F>(self, mapper: F) -> MapStream<Self, F, T>
    where
        Self: Sized,
        F: Fn(T) -> U,
    {
        MapStream { stream: self, mapper, _phantom: std::marker::PhantomData }
    }

    /// Filter elements based on a predicate
    fn filter<F>(self, predicate: F) -> FilterStream<Self, F>
    where
        Self: Sized,
        F: Fn(&T) -> bool,
    {
        FilterStream { stream: self, predicate }
    }

    /// Transform each element into a stream and flatten
    fn flat_map<U, F>(self, mapper: F) -> FlatMapStream<Self, F, T, U>
    where
        Self: Sized,
        F: Fn(T) -> Box<dyn Stream<U>>,
    {
        FlatMapStream { stream: self, mapper, current_inner: None, _phantom: std::marker::PhantomData }
    }

    /// Reduce stream to a single accumulated value
    fn reduce<U, F>(self, initial: U, accumulator: F) -> ReduceFuture<Self, T, U, F>
    where
        Self: Sized,
        F: Fn(U, T) -> U,
    {
        ReduceFuture {
            stream: self,
            accumulator: Some((initial, accumulator)),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Take first N elements
    fn take(self, count: usize) -> TakeStream<Self>
    where
        Self: Sized,
    {
        TakeStream { stream: self, remaining: count }
    }

    /// Skip first N elements
    fn drop(self, count: usize) -> DropStream<Self>
    where
        Self: Sized,
    {
        DropStream { stream: self, remaining: count }
    }

    /// Take elements while predicate is true
    fn take_while<F>(self, predicate: F) -> TakeWhileStream<Self, F>
    where
        Self: Sized,
        F: Fn(&T) -> bool,
    {
        TakeWhileStream { stream: self, predicate }
    }

    /// Buffer elements into lists of specified size
    fn buffer(self, size: usize) -> BufferStream<Self, T>
    where
        Self: Sized,
    {
        BufferStream {
            stream: self,
            size,
            buffer: Vec::new(),
        }
    }

    /// Subscribe to stream events
    fn subscribe<F, E, C>(self, on_next: F, on_error: E, on_complete: C) -> Subscription
    where
        Self: Sized + Send + 'static,
        F: Fn(T) + Send + 'static,
        E: Fn(Box<dyn std::error::Error>) + Send + 'static,
        C: Fn() + Send + 'static,
    {
        let subscription = Subscription::new();
        let subscription_clone = subscription.clone();

        tokio::spawn(async move {
            let mut stream = self;
            loop {
                if !subscription_clone.is_subscribed() {
                    break;
                }

                match stream.poll_next() {
                    Poll::Ready(Some(value)) => on_next(value),
                    Poll::Ready(None) => {
                        on_complete();
                        break;
                    }
                    Poll::Pending => {
                        // For truly async streams, wait a bit before polling again
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                }
            }
        });

        subscription
    }

    /// Poll for the next value (internal method)
    fn poll_next(&mut self) -> Poll<Option<T>>;
}

/// Stream that transforms elements using a mapper function
pub struct MapStream<S, F, T> {
    stream: S,
    mapper: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<S, F, T, U> Stream<U> for MapStream<S, F, T>
where
    S: Stream<T>,
    F: Fn(T) -> U + Send,
    T: Sized + Send,
    U: Sized + Send,
{
    fn poll_next(&mut self) -> Poll<Option<U>> {
        match self.stream.poll_next() {
            Poll::Ready(Some(value)) => Poll::Ready(Some((self.mapper)(value))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream that filters elements based on a predicate
pub struct FilterStream<S, F> {
    stream: S,
    predicate: F,
}

impl<S, F, T> Stream<T> for FilterStream<S, F>
where
    S: Stream<T>,
    F: Fn(&T) -> bool + Send,
    T: Send,
{
    fn poll_next(&mut self) -> Poll<Option<T>> {
        loop {
            match self.stream.poll_next() {
                Poll::Ready(Some(value)) => {
                    if (self.predicate)(&value) {
                        return Poll::Ready(Some(value));
                    }
                    // Continue polling for next value
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Stream that flattens nested streams
pub struct FlatMapStream<S, F, T, U> {
    stream: S,
    mapper: F,
    current_inner: Option<Box<dyn Stream<U>>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<S, F, T, U> Stream<U> for FlatMapStream<S, F, T, U>
where
    S: Stream<T>,
    F: Fn(T) -> Box<dyn Stream<U>> + Send,
    T: Sized + Send,
    U: Sized + Send,
{
    fn poll_next(&mut self) -> Poll<Option<U>> {
        loop {
            // If we have an active inner stream, poll it first
            if let Some(ref mut inner) = self.current_inner {
                match inner.poll_next() {
                    Poll::Ready(Some(value)) => return Poll::Ready(Some(value)),
                    Poll::Ready(None) => {
                        self.current_inner = None;
                        // Continue to get next outer value
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            // Get next outer value and create inner stream
            match self.stream.poll_next() {
                Poll::Ready(Some(value)) => {
                    self.current_inner = Some((self.mapper)(value));
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Stream that takes first N elements
pub struct TakeStream<S> {
    stream: S,
    remaining: usize,
}

impl<S, T> Stream<T> for TakeStream<S>
where
    S: Stream<T>,
    T: Send,
{
    fn poll_next(&mut self) -> Poll<Option<T>> {
        if self.remaining == 0 {
            return Poll::Ready(None);
        }

        match self.stream.poll_next() {
            Poll::Ready(Some(value)) => {
                self.remaining -= 1;
                Poll::Ready(Some(value))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream that skips first N elements
pub struct DropStream<S> {
    stream: S,
    remaining: usize,
}

impl<S, T> Stream<T> for DropStream<S>
where
    S: Stream<T>,
    T: Send,
{
    fn poll_next(&mut self) -> Poll<Option<T>> {
        while self.remaining > 0 {
            match self.stream.poll_next() {
                Poll::Ready(Some(_)) => {
                    self.remaining -= 1;
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }

        self.stream.poll_next()
    }
}

/// Stream that takes elements while predicate is true
pub struct TakeWhileStream<S, F> {
    stream: S,
    predicate: F,
}

impl<S, F, T> Stream<T> for TakeWhileStream<S, F>
where
    S: Stream<T>,
    F: Fn(&T) -> bool + Send,
    T: Send,
{
    fn poll_next(&mut self) -> Poll<Option<T>> {
        match self.stream.poll_next() {
            Poll::Ready(Some(value)) => {
                if (self.predicate)(&value) {
                    Poll::Ready(Some(value))
                } else {
                    Poll::Ready(None)
                }
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream that buffers elements into lists
pub struct BufferStream<S, T> {
    stream: S,
    size: usize,
    buffer: Vec<T>,
}

impl<S, T> Stream<Vec<T>> for BufferStream<S, T>
where
    S: Stream<T>,
    T: Send,
    T: Clone,
{
    fn poll_next(&mut self) -> Poll<Option<Vec<T>>> {
        loop {
            match self.stream.poll_next() {
                Poll::Ready(Some(value)) => {
                    self.buffer.push(value);
                    if self.buffer.len() >= self.size {
                        let result = self.buffer.clone();
                        self.buffer.clear();
                        return Poll::Ready(Some(result));
                    }
                }
                Poll::Ready(None) => {
                    if self.buffer.is_empty() {
                        return Poll::Ready(None);
                    } else {
                        let result = self.buffer.clone();
                        self.buffer.clear();
                        return Poll::Ready(Some(result));
                    }
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Future that reduces a stream to a single value
pub struct ReduceFuture<S, T, U, F> {
    stream: S,
    accumulator: Option<(U, F)>,
    _phantom: std::marker::PhantomData<T>,
}

impl<S, T, U, F> Future for ReduceFuture<S, T, U, F>
where
    S: Stream<T>,
    F: Fn(U, T) -> U + Send,
    T: Sized + Send,
    U: Sized + Send,
{
    type Output = U;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Project the pinned fields
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        
        if let Some((mut acc, accumulator)) = this.accumulator.take() {
            loop {
                match this.stream.poll_next() {
                    Poll::Ready(Some(value)) => {
                        acc = accumulator(acc, value);
                    }
                    Poll::Ready(None) => return Poll::Ready(acc),
                    Poll::Pending => {
                        this.accumulator = Some((acc, accumulator));
                        return Poll::Pending;
                    }
                }
            }
        }
        Poll::Pending
    }
}

/// Concrete stream implementation backed by an async iterator
pub struct StreamImpl<T> {
    iterator: Box<dyn AsyncIterator<Item = T>>,
}

impl<T> StreamImpl<T> {
    pub fn new(iterator: Box<dyn AsyncIterator<Item = T>>) -> Self {
        Self { iterator }
    }
}

impl<T> Stream<T> for StreamImpl<T>
where
    T: Send,
{
    fn poll_next(&mut self) -> Poll<Option<T>> {
        self.iterator.poll_next()
    }
}

/// Stream builders for creating streams from various sources
pub struct StreamBuilder;

impl StreamBuilder {
    /// Create a stream from an iterator
    pub fn from_iter<T: 'static, I>(iter: I) -> StreamImpl<T>
    where
        I: Iterator<Item = T> + 'static + Send,
    {
        // Convert iterator to async iterator
        let async_iter = AsyncIteratorWrapper::new(iter);
        StreamImpl::new(Box::new(async_iter))
    }

    /// Create a stream that emits a single value
    pub fn just<T: 'static + Send>(value: T) -> StreamImpl<T> {
        let iter = std::iter::once(value);
        Self::from_iter(iter)
    }

    /// Create an empty stream
    pub fn empty<T: 'static>() -> StreamImpl<T> {
        let iter = std::iter::empty();
        Self::from_iter(iter)
    }

    /// Create a stream that emits values at regular intervals
    pub fn interval(period: Duration) -> StreamImpl<u64> {
        let iter = IntervalIterator::new(period);
        StreamImpl::new(Box::new(iter))
    }
}

/// Wrapper to convert Iterator to AsyncIterator
struct AsyncIteratorWrapper<I> {
    iter: I,
}

impl<I> AsyncIteratorWrapper<I> {
    fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I, T> AsyncIterator for AsyncIteratorWrapper<I>
where
    I: Iterator<Item = T> + Send,
{
    type Item = T;

    fn poll_next(&mut self) -> Poll<Option<Self::Item>> {
        match self.iter.next() {
            Some(value) => Poll::Ready(Some(value)),
            None => Poll::Ready(None),
        }
    }
}

/// Async iterator trait (simplified)
pub trait AsyncIterator: Send {
    type Item;
    fn poll_next(&mut self) -> Poll<Option<Self::Item>>;
}

/// Interval iterator for periodic emissions
struct IntervalIterator {
    period: Duration,
    count: u64,
    last_emit: std::time::Instant,
}

impl IntervalIterator {
    fn new(period: Duration) -> Self {
        Self {
            period,
            count: 0,
            last_emit: std::time::Instant::now(),
        }
    }
}

impl AsyncIterator for IntervalIterator {
    type Item = u64;

    fn poll_next(&mut self) -> Poll<Option<Self::Item>> {
        if self.last_emit.elapsed() >= self.period {
            self.last_emit = std::time::Instant::now();
            let count = self.count;
            self.count += 1;
            Poll::Ready(Some(count))
        } else {
            Poll::Pending
        }
    }
}

/// Backpressure buffer for flow control
pub struct BackpressureBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
    demand: usize,
}

impl<T> BackpressureBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::new(),
            capacity,
            demand: 0,
        }
    }

    pub fn offer(&mut self, value: T) -> bool {
        if self.buffer.len() >= self.capacity {
            false
        } else {
            self.buffer.push_back(value);
            true
        }
    }

    pub fn poll(&mut self) -> Option<T> {
        self.buffer.pop_front()
    }

    pub fn request(&mut self, count: usize) {
        self.demand += count;
    }

    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_stream_just() {
        let stream = StreamBuilder::just(42);
        let values = Arc::new(Mutex::new(Vec::new()));

        let subscription = stream.subscribe(
            {
                let values = Arc::clone(&values);
                move |value| values.lock().unwrap().push(value)
            },
            |_| {},
            || {},
        );

        // Wait a bit for async processing
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(*values.lock().unwrap(), vec![42]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_map() {
        let stream = StreamBuilder::just(21).map(|x| x * 2);
        let values = Arc::new(Mutex::new(Vec::new()));

        let subscription = stream.subscribe(
            {
                let values = Arc::clone(&values);
                move |value| values.lock().unwrap().push(value)
            },
            |_| {},
            || {},
        );

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(*values.lock().unwrap(), vec![42]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_filter() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5].into_iter())
            .filter(|&x| x % 2 == 0);

        let values = Arc::new(Mutex::new(Vec::new()));

        let subscription = stream.subscribe(
            {
                let values = Arc::clone(&values);
                move |value| values.lock().unwrap().push(value)
            },
            |_| {},
            || {},
        );

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(*values.lock().unwrap(), vec![2, 4]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_take() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5].into_iter())
            .take(3);

        let values = Arc::new(Mutex::new(Vec::new()));

        let subscription = stream.subscribe(
            {
                let values = Arc::clone(&values);
                move |value| values.lock().unwrap().push(value)
            },
            |_| {},
            || {},
        );

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(*values.lock().unwrap(), vec![1, 2, 3]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_backpressure_buffer() {
        let mut buffer = BackpressureBuffer::new(3);

        assert!(buffer.offer(1));
        assert!(buffer.offer(2));
        assert!(buffer.offer(3));
        assert!(!buffer.offer(4)); // Buffer full

        assert_eq!(buffer.poll(), Some(1));
        assert_eq!(buffer.poll(), Some(2));
        assert!(buffer.offer(4)); // Now can offer

        assert_eq!(buffer.size(), 2);
        assert!(!buffer.is_empty());
    }
}