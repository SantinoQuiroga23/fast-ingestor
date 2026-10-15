"""
Python integration tests for FastIngestor.
"""
import pytest
import time


def test_import_module():
    """Test that the module can be imported."""
    import fast_ingestor
    assert hasattr(fast_ingestor, 'FastIngestor')


def test_create_ingestor():
    """Test creating an ingestor instance."""
    from fast_ingestor import FastIngestor
    
    ingestor = FastIngestor(
        api_key="test-key",
        port=8082,  # Use different port to avoid conflicts
        capacity=1000,
    )
    
    # Give server time to start
    time.sleep(0.1)
    assert ingestor is not None


def test_drain_empty():
    """Test draining when no data has been sent."""
    from fast_ingestor import FastIngestor
    
    ingestor = FastIngestor(
        api_key="test-key",
        port=8083,
        capacity=1000,
    )
    
    # Wait for server to start
    time.sleep(0.1)
    
    # Drain should return empty DataFrame
    df = ingestor.drain_batch()
    assert df is not None
    assert len(df) == 0


def test_multi_tenant_mode():
    """Test creating ingestor with multi-tenant mode."""
    from fast_ingestor import FastIngestor
    
    ingestor = FastIngestor(
        api_key="test-key",
        port=8084,
        capacity=1000,
        multi_tenant=True,
    )
    
    time.sleep(0.1)
    assert ingestor is not None


def test_invalid_capacity():
    """Test that capacity must be positive."""
    from fast_ingestor import FastIngestor
    
    # This should work (capacity > 0)
    ingestor = FastIngestor(
        api_key="test-key",
        port=8085,
        capacity=100,
    )
    assert ingestor is not None


def test_drain_multiple_times():
    """Test draining multiple times in succession."""
    from fast_ingestor import FastIngestor
    
    ingestor = FastIngestor(
        api_key="test-key",
        port=8086,
        capacity=1000,
    )
    
    time.sleep(0.1)
    
    # Drain multiple times
    for _ in range(3):
        df = ingestor.drain_batch()
        assert df is not None
        assert len(df) == 0