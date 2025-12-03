"""
Tests for Channel Blocking Behavior

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

from runtime.channels import Channel


class TestBlockingSend:
    """Tests for blocking send behavior."""
    
    def test_send_blocks_when_full(self):
        """Test that send blocks when bounded channel is full."""
        sender, receiver = Channel.new(capacity=2)
        
        # Fill channel
        sender.send(1)
        sender.send(2)
        
        # Track if send completed
        send_completed = []
        
        def send_blocking():
            sender.send(3)  # Should block
            send_completed.append(True)
        
        # Start sender thread
        t = threading.Thread(target=send_blocking)
        t.start()
        
        # Give thread time to block
        time.sleep(0.1)
        
        # Send should still be blocked
        assert len(send_completed) == 0
        
        # Receive one value to unblock
        val1 = receiver.receive()
        assert val1 == 1
        
        # Wait for send to complete
        t.join(timeout=1.0)
        
        # Send should have completed
        assert len(send_completed) == 1
        
        # Now receive the remaining values
        val2 = receiver.receive()
        val3 = receiver.receive()
        assert sorted([val2, val3]) == [2, 3]
    
    def test_send_does_not_block_on_unbounded(self):
        """Test that send never blocks on unbounded channel."""
        sender, receiver = Channel.new()  # Unbounded
        
        # Send many values quickly
        for i in range(1000):
            sender.send(i)
        
        # All sends should complete without blocking
        assert receiver.len() == 1000


class TestBlockingReceive:
    """Tests for blocking receive behavior."""
    
    def test_receive_blocks_when_empty(self):
        """Test that receive blocks when channel is empty."""
        sender, receiver = Channel.new()
        
        # Track if receive completed
        receive_completed = []
        received_value = []
        
        def receive_blocking():
            value = receiver.receive()  # Should block
            received_value.append(value)
            receive_completed.append(True)
        
        # Start receiver thread
        t = threading.Thread(target=receive_blocking)
        t.start()
        
        # Give thread time to block
        time.sleep(0.1)
        
        # Receive should still be blocked
        assert len(receive_completed) == 0
        
        # Send value to unblock
        sender.send(42)
        
        # Wait for receive to complete
        t.join(timeout=1.0)
        
        # Receive should have completed
        assert len(receive_completed) == 1
        assert received_value[0] == 42
    
    def test_receive_does_not_block_with_buffered(self):
        """Test that receive doesn't block when values buffered."""
        sender, receiver = Channel.new()
        
        # Buffer values
        for i in range(10):
            sender.send(i)
        
        # Receive should not block
        for i in range(10):
            value = receiver.receive()
            assert value == i


class TestReceiveTimeout:
    """Tests for receive_timeout."""
    
    def test_receive_timeout_success(self):
        """Test receive_timeout returns value if available."""
        sender, receiver = Channel.new()
        
        sender.send(42)
        
        value = receiver.receive_timeout(timeout=1.0)
        assert value == 42
    
    def test_receive_timeout_expires(self):
        """Test receive_timeout returns None on timeout."""
        sender, receiver = Channel.new()
        
        start = time.time()
        value = receiver.receive_timeout(timeout=0.2)
        elapsed = time.time() - start
        
        assert value is None
        assert 0.15 < elapsed < 0.3  # Allow some timing slack
    
    def test_receive_timeout_with_delayed_send(self):
        """Test receive_timeout with value sent during wait."""
        sender, receiver = Channel.new()
        
        def delayed_send():
            time.sleep(0.1)
            sender.send("delayed")
        
        t = threading.Thread(target=delayed_send)
        t.start()
        
        value = receiver.receive_timeout(timeout=1.0)
        
        t.join()
        
        assert value == "delayed"


class TestMultithreading:
    """Tests for multithreaded usage."""
    
    def test_concurrent_sends(self):
        """Test multiple threads sending concurrently."""
        sender, receiver = Channel.new()
        
        def send_numbers(start, count):
            for i in range(start, start + count):
                sender.send(i)
        
        # Spawn 5 threads, each sending 10 numbers
        threads = []
        for i in range(5):
            t = threading.Thread(target=send_numbers, args=(i * 10, 10))
            threads.append(t)
            t.start()
        
        # Wait for all sends
        for t in threads:
            t.join()
        
        # Should have received 50 numbers
        received = []
        for _ in range(50):
            received.append(receiver.receive())
        
        # All numbers should be present (order doesn't matter)
        assert sorted(received) == list(range(50))
    
    def test_concurrent_receives(self):
        """Test multiple threads receiving concurrently."""
        sender, receiver = Channel.new()
        
        # Send values
        for i in range(50):
            sender.send(i)
        
        received = []
        lock = threading.Lock()
        
        def receive_numbers(count):
            for _ in range(count):
                value = receiver.receive()
                with lock:
                    received.append(value)
        
        # Spawn 5 threads, each receiving 10 numbers
        threads = []
        for _ in range(5):
            t = threading.Thread(target=receive_numbers, args=(10,))
            threads.append(t)
            t.start()
        
        # Wait for all receives
        for t in threads:
            t.join()
        
        # All numbers should be received
        assert sorted(received) == list(range(50))


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
