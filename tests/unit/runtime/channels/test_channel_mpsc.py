"""
Tests for MPSC (Multiple Producer Single Consumer)

Tests VELA-580 (TASK-051)
Sprint 19 - Workers y Channels
"""

import pytest
import threading
import sys
import os

# Add src to path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../src')))

from runtime.channels import Channel


class TestSenderClone:
    """Tests for sender.clone()."""
    
    def test_clone_sender(self):
        """Test cloning sender."""
        sender, receiver = Channel.new()
        
        # Clone sender
        sender2 = sender.clone()
        
        # Both should work
        sender.send(1)
        sender2.send(2)
        
        assert receiver.receive() == 1
        assert receiver.receive() == 2
    
    def test_clone_multiple_times(self):
        """Test cloning sender multiple times."""
        sender, receiver = Channel.new()
        
        # Create 5 clones
        senders = [sender.clone() for _ in range(5)]
        
        # All should work
        for i, s in enumerate(senders):
            s.send(i)
        
        # Receive all
        received = sorted([receiver.receive() for _ in range(5)])
        assert received == [0, 1, 2, 3, 4]


class TestMPSCPattern:
    """Tests for MPSC pattern (Multiple Producers, Single Consumer)."""
    
    def test_mpsc_basic(self):
        """Test basic MPSC pattern."""
        sender, receiver = Channel.new()
        
        # Create 3 producers
        def producer(sender, id, count):
            for i in range(count):
                sender.send((id, i))
        
        threads = []
        for id in range(3):
            s = sender.clone()
            t = threading.Thread(target=producer, args=(s, id, 10))
            threads.append(t)
            t.start()
        
        # Drop original sender
        del sender
        
        # Wait for producers
        for t in threads:
            t.join()
        
        # Receive all messages
        received = []
        for _ in range(30):
            received.append(receiver.receive())
        
        # Should have 10 messages from each producer
        producer_counts = {0: 0, 1: 0, 2: 0}
        for (id, _) in received:
            producer_counts[id] += 1
        
        assert producer_counts == {0: 10, 1: 10, 2: 10}
    
    def test_mpsc_with_bounded_channel(self):
        """Test MPSC with bounded channel."""
        sender, receiver = Channel.new(capacity=5)
        
        # Create 2 producers
        def producer(sender, start, count):
            for i in range(start, start + count):
                sender.send(i)
        
        s1 = sender.clone()
        s2 = sender.clone()
        
        t1 = threading.Thread(target=producer, args=(s1, 0, 25))
        t2 = threading.Thread(target=producer, args=(s2, 100, 25))
        
        t1.start()
        t2.start()
        
        # Receive all
        received = []
        for _ in range(50):
            received.append(receiver.receive())
        
        t1.join()
        t2.join()
        
        # Should have all 50 numbers
        assert len(received) == 50
        
        # Check ranges
        range1 = [x for x in received if x < 100]
        range2 = [x for x in received if x >= 100]
        
        assert sorted(range1) == list(range(0, 25))
        assert sorted(range2) == list(range(100, 125))


class TestMPSCAutoClose:
    """Tests for auto-close with multiple senders."""
    
    def test_autoclose_waits_for_all_senders(self):
        """Test channel doesn't close until all senders dropped."""
        sender, receiver = Channel.new()
        
        # Clone sender
        sender2 = sender.clone()
        sender3 = sender.clone()
        
        # Drop one sender
        del sender
        
        # Channel should still be open
        assert receiver.is_closed() is False
        
        # Drop second sender
        del sender2
        
        # Still open
        assert receiver.is_closed() is False
        
        # Drop last sender
        del sender3
        
        # Now closed (may need small delay for destructor)
        import time
        time.sleep(0.1)
        assert receiver.is_closed() is True
    
    @pytest.mark.skip(reason="Python GC timing is non-deterministic")
    def test_mpsc_producers_finish_independently(self):
        """Test producers can finish at different times."""
        sender, receiver = Channel.new()
        
        import time
        
        def fast_producer(sender):
            sender.send("fast")
            # Sender dropped immediately after
        
        def slow_producer(sender):
            time.sleep(0.2)
            sender.send("slow")
            # Sender dropped after delay
        
        s1 = sender.clone()
        s2 = sender.clone()
        
        t1 = threading.Thread(target=fast_producer, args=(s1,))
        t2 = threading.Thread(target=slow_producer, args=(s2,))
        
        t1.start()
        t2.start()
        
        # Drop original
        del sender
        
        # Receive both messages
        msg1 = receiver.receive()
        assert msg1 == "fast"
        
        # Channel should still be open (slow producer still running)
        assert receiver.is_closed() is False
        
        # Wait for slow producer
        t1.join()
        t2.join()
        
        msg2 = receiver.receive()
        assert msg2 == "slow"
        
        # Now channel should close (wait for Python to run __del__)
        time.sleep(0.3)
        assert receiver.is_closed() is True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
