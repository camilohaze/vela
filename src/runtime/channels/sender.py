"""
Sender - Channel Send Operations

Implementación de: VELA-580 (TASK-051)
Sprint 19 - Workers y Channels

Sender<T> maneja todas las operaciones de envío:
- send(): Blocking
- try_send(): Non-blocking
- send_async(): Async
- clone(): MPSC support
- close(): Manual close

Thread-safe: Todas las operaciones son thread-safe.
"""

from typing import TypeVar, Generic, Optional
from .channel_state import ChannelState
from .exceptions import ChannelClosedError, ChannelFullError


T = TypeVar('T')


class Sender(Generic[T]):
    """
    Sender side of a channel.
    
    Provides thread-safe send operations.
    Supports cloning for MPSC (Multiple Producer Single Consumer).
    """
    
    def __init__(self, state: ChannelState[T]):
        """
        Initialize sender.
        
        Args:
            state: Shared channel state
        """
        self._state = state
        self._closed = False  # Local close flag
        self._count_decremented = False  # Track if count already decremented
    
    def send(self, value: T) -> None:
        """
        Send value to channel (blocking).
        
        Blocks if channel is full (bounded channels).
        
        Args:
            value: Value to send
            
        Raises:
            ChannelClosedError: If channel is closed
            
        Thread-safe.
        """
        with self._state.lock:
            # Check if channel or sender closed
            if self._closed or self._state.closed:
                raise ChannelClosedError("Cannot send to closed channel")
            
            # For bounded channels: wait if full
            while self._state.is_full() and not self._state.closed:
                self._state.not_full.wait()
            
            # Check again after waking up
            if self._state.closed:
                raise ChannelClosedError("Channel closed while waiting to send")
            
            # Add to buffer
            self._state.buffer.append(value)
            
            # Notify waiting receivers
            self._state.not_empty.notify()
    
    def try_send(self, value: T) -> bool:
        """
        Try to send value without blocking.
        
        Returns immediately if channel is full.
        
        Args:
            value: Value to send
            
        Returns:
            True if sent successfully, False if channel full
            
        Raises:
            ChannelClosedError: If channel is closed
            
        Thread-safe.
        """
        with self._state.lock:
            # Check if closed
            if self._closed or self._state.closed:
                raise ChannelClosedError("Cannot send to closed channel")
            
            # Check if full
            if self._state.is_full():
                return False
            
            # Add to buffer
            self._state.buffer.append(value)
            
            # Notify waiting receivers
            self._state.not_empty.notify()
            
            return True
    
    async def send_async(self, value: T) -> None:
        """
        Send value asynchronously.
        
        For integration with async/await.
        Currently blocks (TODO: make truly async in future).
        
        Args:
            value: Value to send
            
        Raises:
            ChannelClosedError: If channel is closed
            
        Thread-safe.
        """
        # For now, just call blocking send
        # TODO: Implement true async send in future Sprint
        self.send(value)
    
    def clone(self) -> 'Sender[T]':
        """
        Clone sender for MPSC (Multiple Producer Single Consumer).
        
        Returns new Sender sharing same channel.
        Increments sender reference count.
        
        Returns:
            New Sender instance
            
        Thread-safe.
        """
        # Increment sender count
        self._state.increment_sender_count()
        
        # Create new sender with same state
        return Sender(self._state)
    
    def close(self) -> None:
        """
        Close sender.
        
        Marks sender as closed locally.
        Decrements sender count (may auto-close channel).
        
        Idempotent: multiple closes OK.
        
        Thread-safe.
        """
        if not self._closed:
            self._closed = True
            # Decrement sender count (may trigger auto-close)
            if not self._count_decremented:
                self._count_decremented = True
                self._state.decrement_sender_count()
    
    def is_closed(self) -> bool:
        """
        Check if sender is closed.
        
        Returns:
            True if sender or channel closed
            
        Thread-safe.
        """
        with self._state.lock:
            return self._closed or self._state.closed
    
    def __del__(self):
        """
        Destructor: Decrement sender count when dropped.
        
        Auto-closes channel when last sender is dropped.
        """
        if hasattr(self, '_state') and hasattr(self, '_count_decremented'):
            if not self._count_decremented:
                self._count_decremented = True
                self._state.decrement_sender_count()
    
    def __repr__(self) -> str:
        """String representation for debugging."""
        return f"Sender(closed={self._closed}, state={self._state})"
