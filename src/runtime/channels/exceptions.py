"""
Channel Exceptions

Implementación de: VELA-580 (TASK-051)
Sprint 19 - Workers y Channels

Excepciones específicas para operaciones de Channel<T>.
"""


class ChannelError(Exception):
    """Base exception for channel operations."""
    pass


class ChannelClosedError(ChannelError):
    """Raised when operating on a closed channel."""
    pass


class ChannelFullError(ChannelError):
    """Raised when trying to send to a full bounded channel (non-blocking)."""
    pass


class ChannelEmptyError(ChannelError):
    """Raised when trying to receive from empty channel (non-blocking)."""
    pass
