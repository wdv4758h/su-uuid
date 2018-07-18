import pytest
import uuid as py_uuid
import uuid_rpy as rs_uuid


@pytest.mark.benchmark(group='uuid1')
def test_uuid1_rs(benchmark):
    benchmark(rs_uuid.uuid1)

@pytest.mark.benchmark(group='uuid1')
def test_uuid1_py(benchmark):
    benchmark(py_uuid.uuid1)
