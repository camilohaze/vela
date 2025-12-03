"""
Tests for Channel Close Semantics

Tests VELA-580 (TASK-051)
Sprint 19 - Workers y Channels
"""

import pytest
import threading
import time
import sys
import os

# Add src to path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../src')))

from runtime.channels import Channel, ChannelClosedError


class TestManualClose:
    """Tests for manual close operations."""
    
    def test_receiver_close(self):
        """Test receiver can close channel."""
        sender, receiver = Channel.new()
        
        receiver.close()
        
        assert receiver.is_closed() is True
    
    def test_sender_close(self):
        """Test sender close marks sender as closed."""
        sender, receiver = Channel.new()
        
        sender.close()
        
        assert sender.is_closed() is True
    
    def test_close_is_idempotent(self):
        """Test that closing multiple times is OK."""
        sender, receiver = Channel.new()
        
        receiver.close()
        receiver.close()  # Second close should not error
        receiver.close()  # Third close should not error
        
        assert receiver.is_closed() is True
    
    def test_close_wakes_blocked_sender(self):
        """Test that close wakes blocked senders."""
        sender, receiver = Channel.new(capacity=1)
        
        # Fill channel
        sender.send(1)
        
        # Track if send raised error
        errors = []
        
        def send_blocking():
            try:
                sender.send(2)  # Should block
            except ChannelClosedError as e:
                errors.append(e)
        
        t = threading.Thread(target=send_blocking)
        t.start()
        
        # Give time to block
        time.sleep(0.1)
        
        # Close channel
        receiver.close()
        
        # Wait for thread
        t.join(timeout=1.0)
        
        # Should have raised ChannelClosedError
        assert len(errors) == 1
    
    def test_close_wakes_blocked_receiver(self):
        """Test that close wakes blocked receivers."""
        sender, receiver = Channel.new()
        
        # Track receive result
        results = []
        
        def receive_blocking():
            value = receiver.receive()  # Should block
            results.append(value)
        
        t = threading.Thread(target=receive_blocking)
        t.start()
        
        # Give time to block
        time.sleep(0.1)
        
        # Close channel
        sender.close()
        
        # Wait for thread
        t.join(timeout=1.0)
        
        # Should have returned None
        assert results == [None]


class TestAutoClose:
    """Tests for auto-close when senders dropped."""
    
    def test_autoclose_when_sender_dropped(self):
        """Test channel auto-closes when sender is dropped."""
        sender, receiver = Channel.new()
        
        # Drop sender (delete reference)
        del sender
        
        # Give Python time to run destructor
        time.sleep(0.1)
        
        # Channel should be closed
        assert receiver.is_closed() is True
    
    def test_autoclose_with_cloned_senders(self):
        """Test channel stays open while cloned senders exist."""
        sender, receiver = Channel.new()
        
        # Clone sender
        sender2 = sender.clone()
        
        # Drop original
        del sender
        time.sleep(0.1)
        
        # Channel should still be open (sender2 exists)
        assert receiver.is_closed() is False
        
        # Drop clone
        del sender2
        time.sleep(0.1)
        
        # Now channel should be closed
        assert receiver.is_closed() is True
    
    def test_autoclose_wakes_receiver(self):
        """Test that auto-close wakes blocked receiver."""
        sender, receiver = Channel.new()
        
        # Track receive result
        results = []
        
        def receive_blocking():
            value = receiver.receive()
            results.append(value)
        
        t = threading.Thread(target=receive_blocking)
        t.start()
        
        # Give time to block
        time.sleep(0.1)
        
        # Drop sender (triggers auto-close)
        del sender
        
        # Wait for thread
        t.join(timeout=1.0)
        
        # Should have returned None
        assert results == [None]


class TestBufferedAfterClose:
    """Tests for buffered messages after close."""
    
    def test_receive_buffered_after_manual_close(self):
        """Test buffered messages remain after manual close."""
        sender, receiver = Channel.new()
        
        # Send multiple values
        for i in range(5):
            sender.send(i)
        
        # Close
        receiver.close()
        
        # Can still receive all buffered
        for i in range(5):
            value = receiver.receive()
            assert value == i
        
        # Empty now
        value = receiver.receive()
        assert value is None
    
    def test_receive_buffered_after_autoclose(self):
        """Test buffered messages remain after auto-close."""
        sender, receiver = Channel.new()
        
        # Send values
        sender.send(1)
        sender.send(2)
        sender.send(3)
        
        # Drop sender (auto-close)
        del sender
        time.sleep(0.1)
        
        # Can still receive buffered
        assert receiver.receive() == 1
        assert receiver.receive() == 2
        assert receiver.receive() == 3
        assert receiver.receive() is None


class TestReceiverIterator:
    """Tests for receiver as iterator."""
    
    def test_receiver_iteration(self):
        """Test iterating over receiver."""
        sender, receiver = Channel.new()
        
        # Send values
        for i in range(5):
            sender.send(i)
        
        # Close sender
        sender.close()
        
        # Iterate over receiver
        received = []
        for value in receiver:
            received.append(value)
        
        assert received == [0, 1, 2, 3, 4]
    
    def test_iteration_with_autoclose(self):
        """Test iteration stops when auto-closed."""
        sender, receiver = Channel.new()
        
        # Send values
        for i in range(3):
            sender.send(i)
        
        # Drop sender
        del sender
        time.sleep(0.1)
        
        # Iterate
        received = list(receiver)
        assert received == [0, 1, 2]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
