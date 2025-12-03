"""
Channel - Factory for Creating Channels

Implementación de: VELA-580 (TASK-051)
Sprint 19 - Workers y Channels

Channel<T> es el factory para crear channels:
- new(capacity): Crea bounded/unbounded channel
- Retorna (Sender<T>, Receiver<T>) tuple

Thread-safe: El factory y los channels son thread-safe.
"""

from typing import TypeVar, Optional, Tuple
from .channel_state import ChannelState
from .sender import Sender
from .receiver import Receiver


T = TypeVar('T')


class Channel:
    """
    Channel factory.
    
    Use Channel.new() to create a new channel.
    Returns (Sender, Receiver) tuple.
    """
    
    @staticmethod
    def new(capacity: Optional[int] = None) -> Tuple[Sender[T], Receiver[T]]:
        """
        Create a new channel.
        
        Args:
            capacity: Max buffer size. None = unbounded.
                     If specified, sender blocks when full.
                     
        Returns:
            (Sender, Receiver) tuple
            
        Examples:
            # Unbounded channel
            sender, receiver = Channel.new()
            
            # Bounded channel (capacity 10)
            sender, receiver = Channel.new(capacity=10)
            
            # MPSC: Clone sender for multiple producers
            sender1 = sender.clone()
            sender2 = sender.clone()
        """
        # Validate capacity
        if capacity is not None and capacity <= 0:
            raise ValueError(f"Capacity must be positive, got {capacity}")
        
        # Create shared state
        state = ChannelState[T](capacity=capacity)
        
        # Create sender and receiver
        sender = Sender(state)
        receiver = Receiver(state)
        
        return (sender, receiver)
    
    @staticmethod
    def unbounded() -> Tuple[Sender[T], Receiver[T]]:
        """
        Create unbounded channel.
        
        Convenience method for Channel.new(capacity=None).
        
        Returns:
            (Sender, Receiver) tuple
        """
        return Channel.new(capacity=None)
    
    @staticmethod
    def bounded(capacity: int) -> Tuple[Sender[T], Receiver[T]]:
        """
        Create bounded channel.
        
        Convenience method for Channel.new(capacity=capacity).
        
        Args:
            capacity: Max buffer size
            
        Returns:
            (Sender, Receiver) tuple
        """
        return Channel.new(capacity=capacity)
