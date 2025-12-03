"""
Tests for Basic Channel Send/Receive Operations

Tests VELA-580 (TASK-051)
Sprint 19 - Workers y Channels
"""

import pytest
import sys
import os

# Add src to path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../src')))

from runtime.channels import Channel, ChannelClosedError


class TestBasicSendReceive:
    """Tests for basic send/receive operations."""
    
    def test_unbounded_send_receive(self):
        """Test basic send/receive on unbounded channel."""
        sender, receiver = Channel.new()
        
        # Send value
        sender.send("hello")
        
        # Receive value
        value = receiver.receive()
        assert value == "hello"
    
    def test_bounded_send_receive(self):
        """Test basic send/receive on bounded channel."""
        sender, receiver = Channel.new(capacity=5)
        
        # Send value
        sender.send(42)
        
        # Receive value
        value = receiver.receive()
        assert value == 42
    
    def test_multiple_sends(self):
        """Test multiple sends and receives."""
        sender, receiver = Channel.new()
        
        # Send multiple values
        for i in range(10):
            sender.send(i)
        
        # Receive all
        for i in range(10):
            value = receiver.receive()
            assert value == i
    
    def test_fifo_ordering(self):
        """Test FIFO ordering of messages."""
        sender, receiver = Channel.new()
        
        values = ["first", "second", "third"]
        for v in values:
            sender.send(v)
        
        for expected in values:
            actual = receiver.receive()
            assert actual == expected
    
    def test_send_after_close(self):
        """Test that send after close raises error."""
        sender, receiver = Channel.new()
        
        receiver.close()
        
        with pytest.raises(ChannelClosedError):
            sender.send("value")
    
    def test_receive_from_closed_empty_channel(self):
        """Test receiving from closed empty channel returns None."""
        sender, receiver = Channel.new()
        
        sender.close()
        
        value = receiver.receive()
        assert value is None
    
    def test_receive_buffered_after_close(self):
        """Test that buffered messages remain after close."""
        sender, receiver = Channel.new()
        
        # Send values
        sender.send(1)
        sender.send(2)
        sender.send(3)
        
        # Close
        receiver.close()
        
        # Can still receive buffered
        assert receiver.receive() == 1
        assert receiver.receive() == 2
        assert receiver.receive() == 3
        assert receiver.receive() is None


class TestTrySendReceive:
    """Tests for non-blocking try_send/try_receive."""
    
    def test_try_send_success(self):
        """Test successful try_send."""
        sender, receiver = Channel.new(capacity=5)
        
        result = sender.try_send("value")
        assert result is True
        
        value = receiver.receive()
        assert value == "value"
    
    def test_try_send_full(self):
        """Test try_send on full channel returns False."""
        sender, receiver = Channel.new(capacity=2)
        
        # Fill channel
        sender.try_send(1)
        sender.try_send(2)
        
        # Try send to full channel
        result = sender.try_send(3)
        assert result is False
    
    def test_try_receive_success(self):
        """Test successful try_receive."""
        sender, receiver = Channel.new()
        
        sender.send("value")
        
        value = receiver.try_receive()
        assert value == "value"
    
    def test_try_receive_empty(self):
        """Test try_receive on empty channel returns None."""
        sender, receiver = Channel.new()
        
        value = receiver.try_receive()
        assert value is None
    
    def test_try_send_after_close(self):
        """Test try_send after close raises error."""
        sender, receiver = Channel.new()
        
        receiver.close()
        
        with pytest.raises(ChannelClosedError):
            sender.try_send("value")


class TestChannelState:
    """Tests for channel state queries."""
    
    def test_is_empty_true(self):
        """Test is_empty on empty channel."""
        sender, receiver = Channel.new()
        
        assert receiver.is_empty() is True
    
    def test_is_empty_false(self):
        """Test is_empty on non-empty channel."""
        sender, receiver = Channel.new()
        
        sender.send("value")
        
        assert receiver.is_empty() is False
    
    def test_len(self):
        """Test len returns buffer size."""
        sender, receiver = Channel.new()
        
        assert receiver.len() == 0
        
        sender.send(1)
        assert receiver.len() == 1
        
        sender.send(2)
        assert receiver.len() == 2
        
        receiver.receive()
        assert receiver.len() == 1
    
    def test_is_closed_false(self):
        """Test is_closed on open channel."""
        sender, receiver = Channel.new()
        
        assert receiver.is_closed() is False
    
    def test_is_closed_true(self):
        """Test is_closed on closed channel."""
        sender, receiver = Channel.new()
        
        receiver.close()
        
        assert receiver.is_closed() is True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
