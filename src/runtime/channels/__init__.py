"""
Runtime Channels Module

Implementación de: VELA-580 (TASK-051)
Sprint 19 - Workers y Channels

Channel<T> proporciona comunicación thread-safe entre workers:
- Sender/Receiver split model (Rust-inspired)
- Bounded/unbounded channels
- Blocking and non-blocking operations
- MPSC support via sender.clone()
- Auto-close when senders dropped

Usage:
    from runtime.channels import Channel
    
    # Create channel
    sender, receiver = Channel.new(capacity=10)
    
    # Send
    sender.send("hello")
    
    # Receive
    msg = receiver.receive()
    
    # MPSC
    sender2 = sender.clone()
"""

from .channel import Channel
from .sender import Sender
from .receiver import Receiver
from .channel_state import ChannelState
from .exceptions import (
    ChannelError,
    ChannelClosedError,
    ChannelFullError,
    ChannelEmptyError
)


__all__ = [
    'Channel',
    'Sender',
    'Receiver',
    'ChannelState',
    'ChannelError',
    'ChannelClosedError',
    'ChannelFullError',
    'ChannelEmptyError',
]
