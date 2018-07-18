import pytest
import uuid as py_uuid
import su_uuid as rs_uuid


@pytest.mark.benchmark(group='uuid3')
def test_uuid3_rs(benchmark):
    benchmark(rs_uuid.uuid3, rs_uuid.NAMESPACE_DNS, "test")

@pytest.mark.benchmark(group='uuid3')
def test_uuid3_py(benchmark):
    benchmark(py_uuid.uuid3, py_uuid.NAMESPACE_DNS, "test")
