import pytest
import uuid as py_uuid
import su_uuid as rs_uuid


@pytest.mark.benchmark(group='uuid5')
def test_uuid5_rs(benchmark):
    benchmark(rs_uuid.uuid5, rs_uuid.NAMESPACE_DNS, "test")

@pytest.mark.benchmark(group='uuid5')
def test_uuid5_py(benchmark):
    benchmark(py_uuid.uuid5, py_uuid.NAMESPACE_DNS, "test")
