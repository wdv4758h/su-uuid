#![feature(proc_macro, specialization, const_fn)]
extern crate pyo3;
extern crate uuid;

use pyo3::prelude::*;
use uuid::Uuid;

#[py::class]
struct PyUuid {
    data: uuid::Uuid,
}

#[py::methods]
impl PyUuid {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|token| {
            PyUuid {
                data: Uuid::new_v4(),
            }
        })
    }

    #[getter]
    pub fn bytes(&self) -> PyResult<Vec<u8>> {
        // FIXME: do not make new vector
        Ok(self.data.as_bytes().to_vec())
    }
}

#[py::proto]
impl pyo3::class::basic::PyObjectProtocol for PyUuid {
    fn __str__(&self) -> PyResult<String> {
        Ok(self.data.hyphenated().to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("UUID('{}')", self.data.hyphenated().to_string()))
    }
}

pub fn register_constants(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("NAMESPACE_DNS",
          py.init(|token| PyUuid { data: uuid::NAMESPACE_DNS }).unwrap())?;
    m.add("NAMESPACE_OID",
          py.init(|token| PyUuid { data: uuid::NAMESPACE_OID }).unwrap())?;
    m.add("NAMESPACE_URL",
          py.init(|token| PyUuid { data: uuid::NAMESPACE_URL }).unwrap())?;
    m.add("NAMESPACE_X500",
          py.init(|token| PyUuid { data: uuid::NAMESPACE_X500 }).unwrap())?;
    Ok(())
}

pub fn register_classes(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyUuid>()?;
    Ok(())
}

#[py::modinit(uuid_rpy)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    register_constants(py, m)?;
    register_classes(py, m)?;

    #[pyfn(m, "uuid3")]
    fn uuid3(py: Python, namespace: &PyUuid, name: &str)
          -> PyResult<Py<PyUuid>> {
        py.init(|token| {
            PyUuid {
                data: Uuid::new_v3(&namespace.data, name),
            }
        })
    }

    #[pyfn(m, "uuid4")]
    fn uuid4(py: Python) -> PyResult<Py<PyUuid>> {
        py.init(|token| {
            PyUuid {
                data: Uuid::new_v4(),
            }
        })
    }

    #[pyfn(m, "uuid5")]
    fn uuid5(py: Python, namespace: &PyUuid, name: &str)
          -> PyResult<Py<PyUuid>> {
        py.init(|token| {
            PyUuid {
                data: Uuid::new_v5(&namespace.data, name),
            }
        })
    }

    Ok(())
}
