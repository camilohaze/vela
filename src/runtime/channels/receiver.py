"""
Receiver - Channel Receive Operations

Implementación de: VELA-580 (TASK-051)
Sprint 19 - Workers y Channels

Receiver<T> maneja todas las operaciones de recepción:
- receive(): Blocking
- try_receive(): Non-blocking
- receive_async(): Async
- receive_timeout(): With timeout
- len(), is_empty(), is_closed(): State queries

Thread-safe: Todas las operaciones son thread-safe.
Only one Receiver per channel (no cloning).
"""

from typing import TypeVar, Generic, Optional
from .channel_state import ChannelState
from .exceptions import ChannelClosedError, ChannelEmptyError


T = TypeVar('T')


class Receiver(Generic[T]):
    """
    Receiver side of a channel.
    
    Provides thread-safe receive operations.
    Cannot be cloned (only one consumer per channel).
    """
    
    def __init__(self, state: ChannelState[T]):
        """
        Initialize receiver.
        
        Args:
            state: Shared channel state
        """
        self._state = state
        self._closed = False  # Local close flag
    
    def receive(self) -> Optional[T]:
        """
        Receive value from channel (blocking).
        
        Blocks if channel is empty.
        Returns None if channel closed and empty.
        
        Returns:
            Received value or None if closed
            
        Thread-safe.
        """
        with self._state.lock:
            # Wait while empty and not closed
            while self._state.is_empty() and not self._state.closed:
                self._state.not_empty.wait()
            
            # If channel closed and empty, return None
            if self._state.is_empty():
                return None
            
            # Remove from buffer
            value = self._state.buffer.popleft()
            
            # Notify waiting senders (for bounded channels)
            self._state.not_full.notify()
            
            return value
    
    def try_receive(self) -> Optional[T]:
        """
        Try to receive value without blocking.
        
        Returns None immediately if channel is empty.
        
        Returns:
            Received value or None if empty
            
        Thread-safe.
        """
        with self._state.lock:
            # If empty, return None
            if self._state.is_empty():
                return None
            
            # Remove from buffer
            value = self._state.buffer.popleft()
            
            # Notify waiting senders
            self._state.not_full.notify()
            
            return value
    
    async def receive_async(self) -> Optional[T]:
        """
        Receive value asynchronously.
        
        For integration with async/await.
        Currently blocks (TODO: make truly async in future).
        
        Returns:
            Received value or None if closed
            
        Thread-safe.
        """
        # For now, just call blocking receive
        # TODO: Implement true async receive in future Sprint
        return self.receive()
    
    def receive_timeout(self, timeout: float) -> Optional[T]:
        """
        Receive with timeout.
        
        Blocks for at most 'timeout' seconds.
        
        Args:
            timeout: Max seconds to wait
            
        Returns:
            Received value or None if timeout/closed
            
        Thread-safe.
        """
        with self._state.lock:
            # Wait with timeout
            if self._state.is_empty() and not self._state.closed:
                self._state.not_empty.wait(timeout)
            
            # After wait, check if still empty
            if self._state.is_empty():
                return None
            
            # Remove from buffer
            value = self._state.buffer.popleft()
            
            # Notify waiting senders
            self._state.not_full.notify()
            
            return value
    
    def len(self) -> int:
        """
        Get number of buffered messages.
        
        Returns:
            Buffer size
            
        Thread-safe.
        """
        return self._state.len()
    
    def is_empty(self) -> bool:
        """
        Check if buffer is empty.
        
        Returns:
            True if no buffered messages
            
        Thread-safe.
        """
        with self._state.lock:
            return self._state.is_empty()
    
    def is_closed(self) -> bool:
        """
        Check if channel is closed.
        
        Returns:
            True if channel closed
            
        Thread-safe.
        """
        with self._state.lock:
            return self._state.closed
    
    def close(self) -> None:
        """
        Close receiver.
        
        Closes entire channel (affects all senders).
        Buffered messages remain available.
        
        Idempotent: multiple closes OK.
        
        Thread-safe.
        """
        self._state.close()
        self._closed = True
    
    def __iter__(self):
        """
        Make receiver iterable.
        
        Allows: for msg in receiver: ...
        """
        return self
    
    def __next__(self) -> T:
        """
        Iterator protocol.
        
        Returns next value or raises StopIteration when closed.
        """
        value = self.receive()
        if value is None:
            raise StopIteration
        return value
    
    def __repr__(self) -> str:
        """String representation for debugging."""
        return f"Receiver(closed={self._closed}, state={self._state})"
