import pytest
import uuid as py_uuid
import uuid_rpy as rs_uuid


@pytest.mark.benchmark(group='uuid4')
def test_uuid4_rs(benchmark):
    benchmark(rs_uuid.uuid4)

@pytest.mark.benchmark(group='uuid4')
def test_uuid4_py(benchmark):
    benchmark(py_uuid.uuid4)
