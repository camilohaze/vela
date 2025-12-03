"""
Channel State - Shared State Between Sender and Receiver

Implementación de: VELA-580 (TASK-051)
Sprint 19 - Workers y Channels

ChannelState mantiene el estado compartido entre Sender y Receiver:
- Buffer de mensajes (deque)
- Locks para thread safety
- Condition variables para blocking
- Sender reference counting para auto-close

Thread-safe: Todos los accesos protegidos por lock.
"""

from typing import TypeVar, Generic, Optional, Deque
from collections import deque
import threading


T = TypeVar('T')


class ChannelState(Generic[T]):
    """
    Shared state between Sender and Receiver.
    
    Thread-safe internal state for Channel<T>.
    Uses locks + condition variables for synchronization.
    """
    
    def __init__(self, capacity: Optional[int] = None):
        """
        Initialize channel state.
        
        Args:
            capacity: Max buffer size. None = unbounded.
        """
        # Buffer (deque with optional max size)
        self.buffer: Deque[T] = deque(maxlen=capacity) if capacity else deque()
        self.capacity = capacity
        
        # State flags
        self.closed = False
        self.sender_count = 1  # Reference counting for senders
        
        # Thread synchronization
        self.lock = threading.Lock()
        self.not_empty = threading.Condition(self.lock)
        self.not_full = threading.Condition(self.lock)
    
    def is_bounded(self) -> bool:
        """Check if channel is bounded."""
        return self.capacity is not None
    
    def is_full(self) -> bool:
        """
        Check if buffer is full (for bounded channels).
        
        Must be called with lock held.
        """
        if not self.is_bounded():
            return False
        return len(self.buffer) >= self.capacity
    
    def is_empty(self) -> bool:
        """
        Check if buffer is empty.
        
        Must be called with lock held.
        """
        return len(self.buffer) == 0
    
    def len(self) -> int:
        """
        Get current buffer size.
        
        Thread-safe.
        """
        with self.lock:
            return len(self.buffer)
    
    def increment_sender_count(self) -> None:
        """
        Increment sender reference count (when cloning sender).
        
        Thread-safe.
        """
        with self.lock:
            self.sender_count += 1
    
    def decrement_sender_count(self) -> bool:
        """
        Decrement sender reference count (when sender is dropped).
        
        Auto-closes channel when count reaches 0.
        
        Returns:
            True if channel was auto-closed (count reached 0)
            
        Thread-safe.
        """
        with self.lock:
            self.sender_count -= 1
            
            if self.sender_count == 0 and not self.closed:
                # Auto-close channel when no more senders
                self.closed = True
                # Wake all waiting receivers
                self.not_empty.notify_all()
                return True
            
            return False
    
    def close(self) -> None:
        """
        Close channel.
        
        Idempotent: multiple closes OK.
        
        Thread-safe.
        """
        with self.lock:
            if not self.closed:
                self.closed = True
                # Wake all waiting threads
                self.not_empty.notify_all()
                self.not_full.notify_all()
    
    def __repr__(self) -> str:
        """String representation for debugging."""
        with self.lock:
            return (
                f"ChannelState("
                f"capacity={self.capacity}, "
                f"buffered={len(self.buffer)}, "
                f"closed={self.closed}, "
                f"senders={self.sender_count})"
            )
