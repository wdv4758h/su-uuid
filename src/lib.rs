#![feature(proc_macro, specialization, const_fn)]
extern crate pyo3;
extern crate uuid;

use pyo3::PyBytes;
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

pub fn register_classes(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyUuid>()?;
    Ok(())
}

#[py::modinit(uuid_rpy)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    register_classes(py, m)?;
    Ok(())
}
