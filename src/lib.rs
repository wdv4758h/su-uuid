#![feature(i128_type)]
#![feature(proc_macro, specialization, const_fn)]

extern crate pyo3;
extern crate uuid;


use pyo3::prelude::*;
use pyo3::ffi;
use std::str::FromStr;


fn clean_uuid_string(string: &str) -> String {
    let patterns: &[_] = &['{', '}'];
    string.replace("urn:", "")
          .replace("uuid:", "")
          .trim_left_matches(patterns)
          .trim_right_matches(patterns)
          .replace("-", "")
          .to_string()
}


#[py::class]
struct UUID {
    py: PyToken,
    data: uuid::Uuid,
}

impl UUID {
    fn get_u128(&self) -> u128 {
        let ptr = self.data.as_bytes() as *const u8 as *const u128;
        // this is fine, the as_bytes() will return &[u8, 16], which is 128 bit
        let value = unsafe { *ptr };
        let ret = u128::from_be(value);
        println!("{}", ret);    // FIXME: remove this will break things ??? WTF
        ret
    }

    fn get_time(&self) -> u128 {
        (((self.data.as_fields().2 as u128 & 0x0fff) << 48) |
         ((self.data.as_fields().1 as u128) << 32) |
         self.data.as_fields().0 as u128)
    }
}

// implement FromPyObject for u128

#[py::methods]
impl UUID {
    #[new]
    #[args(hex="None", bytes="None", bytes_le="None", fields="None", args="*")]
    fn __new__(obj: &PyRawObject,
               hex: Option<Option<&str>>,
               bytes: Option<Option<Vec<u8>>>,    // FIXME: use reference directly
               bytes_le: Option<Option<Vec<u8>>>, // FIXME: use reference directly
               fields: Option<Option<(u32, u16, u16, u8, u8, u64)>>,
               args: &PyTuple)
      -> PyResult<()> {

        let hex = hex.unwrap();
        let bytes = bytes.unwrap();
        let bytes_le = bytes_le.unwrap();
        let fields = fields.unwrap();

        let uuid =
            if let Some(hex) = hex {
                let string = clean_uuid_string(hex);
                if string.len() != 32 {
                    return Err(exc::ValueError.into());
                }
                uuid::Uuid::from_str(&string)
            } else if let Some(bytes) = bytes {
                uuid::Uuid::from_bytes(&bytes)
            } else if let Some(bytes_le) = bytes_le {
                // FIXME: do not create vector
                let slice = bytes_le[..4].iter().rev()
                    .chain(bytes_le[4..6].iter().rev())
                    .chain(bytes_le[6..8].iter().rev())
                    .chain(bytes_le[8..].iter())
                    .map(|n| *n);
                uuid::Uuid::from_bytes(
                    slice.collect::<Vec<_>>().as_slice()).unwrap()
            } else if let Some(fields) = fields {
               uuid::Uuid::from_fields(
                   fields.0,
                   fields.1,
                   fields.2,
                   &[fields.3, fields.4,
                    ((fields.5 >> 40) % 256) as u8,
                    ((fields.5 >> 32) % 256) as u8,
                    ((fields.5 >> 24) % 256) as u8,
                    ((fields.5 >> 16) % 256) as u8,
                    ((fields.5 >>  8) % 256) as u8,
                    ((fields.5 >>  0) % 256) as u8]).unwrap()
            } else {
                if args.is_empty() {
                    return Err(exc::TypeError.into());
                }
                use std::borrow::Borrow;
                let pystring: &PyString = args.get_item(0).try_into().unwrap();
                let cow_string = pystring.to_string().unwrap();
                let string = cow_string.borrow();
                let string = clean_uuid_string(string);
                if string.len() != 32 {
                    return Err(exc::ValueError.into());
                }
                uuid::Uuid::from_str(&string)
            };

        let uuid = match uuid {
            Ok(uuid) => uuid,
            Err(_) => return Err(exc::ValueError.into()),
        };

        obj.init(|token| {
            UUID {
                py: token,
                data: uuid,
            }
        })
    }

    #[getter]
    pub fn bytes(&self) -> PyResult<PyObject> {
        Ok(PyBytes::new(self.py(),
                        self.data.as_bytes()).into())
    }

    #[getter]
    pub fn bytes_le(&self) -> PyResult<PyObject> {
        // FIXME: do not make new vector
        let data = self.data.as_bytes();
        let slice = data[..4].iter().rev()
            .chain(data[4..6].iter().rev())
            .chain(data[6..8].iter().rev())
            .chain(data[8..].iter())
            .map(|n| *n);
        Ok(PyBytes::new(self.py(),
                        slice.collect::<Vec<_>>().as_slice()).into())
    }

    #[getter]
    pub fn clock_seq(&self) -> PyResult<u8> {
        Ok(self.data.as_fields().3[1])
    }

    #[getter]
    pub fn clock_seq_hi_variant(&self) -> PyResult<u8> {
        Ok(self.data.as_fields().3[0])
    }

    #[getter]
    pub fn clock_seq_low(&self) -> PyResult<u8> {
        Ok(self.data.as_fields().3[1])
    }

    #[getter]
    pub fn fields(&self) -> PyResult<(u32, u16, u16, u8, u8, u64)> {
        let time_low = self.data.as_fields().0;
        let time_mid = self.data.as_fields().1;
        let time_hi_version = self.data.as_fields().2;
        let clock_seq_hi_variant = self.data.as_fields().3[0];
        let clock_seq_low = self.data.as_fields().3[1];
        // FIXME: more efficient way ?
        let node = self.data.as_fields().3[2..]
                       .iter().fold(0_u64, |a, &b| { a*256+(b as u64) });
        Ok((time_low, time_mid, time_hi_version,
            clock_seq_hi_variant, clock_seq_low, node))
    }

    #[getter]
    pub fn hex(&self) -> PyResult<String> {
        Ok(self.data.simple().to_string())
    }

    #[getter]
    pub fn int(&self) -> PyResult<PyObject> {
        // FIXME: use a more light weight way to make PyLong for u128
        let num_string = format!("{}", self.get_u128());
        Ok(unsafe {
            PyObject::from_owned_ptr_or_panic(
                self.py(),
                ffi::PyLong_FromString(
                    num_string.as_ptr() as *const i8,
                    0 as *mut *mut i8,
                    0_i32
                )
            )
        })
    }

    #[getter]
    pub fn node(&self) -> PyResult<u64> {
        // FIXME: more efficient way ?
        Ok(self.data.as_fields().3[2..]
               .iter().fold(0_u64, |a, &b| { a*256+(b as u64) }))
    }

    #[getter]
    pub fn time(&self) -> PyResult<PyObject> {
        // 60 bits timestamp
        let num_string = format!("{}\0", self.get_time());
        Ok(unsafe {
            PyObject::from_owned_ptr_or_panic(
                self.py(),
                ffi::PyLong_FromString(
                    num_string.as_ptr() as *const i8,
                    0 as *mut *mut i8,
                    0_i32
                )
            )
        })
    }

    #[getter]
    pub fn time_hi_version(&self) -> PyResult<u16> {
        Ok(self.data.as_fields().2)
    }

    #[getter]
    pub fn time_low(&self) -> PyResult<u32> {
        Ok(self.data.as_fields().0)
    }

    #[getter]
    pub fn time_mid(&self) -> PyResult<u16> {
        Ok(self.data.as_fields().1)
    }

    #[getter]
    pub fn urn(&self) -> PyResult<String> {
        Ok(self.data.urn().to_string())
    }

    #[getter]
    pub fn variant(&self) -> PyResult<String> {
        Ok(format!("{:?}",
                   self.data.get_variant()
                            .unwrap_or(uuid::UuidVariant::RFC4122)))
    }

    #[getter]
    pub fn version(&self) -> PyResult<usize> {
        Ok(self.data.get_version_num())
    }
}

#[py::proto]
impl pyo3::class::basic::PyObjectProtocol for UUID {
    fn __str__(&self) -> PyResult<String> {
        Ok(self.data.hyphenated().to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("UUID('{}')", self.data.hyphenated().to_string()))
    }

    fn __richcmp__(&self, other: &UUID, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.data == other.data),
            CompareOp::Ne => Ok(self.data != other.data),
            // FIXME: other ops
            _ => Ok(false)
        }
    }
}


////////////////////////////////////////
// convient functions to make PyModule
////////////////////////////////////////

pub fn register_constants(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("NAMESPACE_DNS",
          py.init(|token| UUID { py: token, data: uuid::NAMESPACE_DNS }).unwrap())?;
    m.add("NAMESPACE_OID",
          py.init(|token| UUID { py: token, data: uuid::NAMESPACE_OID }).unwrap())?;
    m.add("NAMESPACE_URL",
          py.init(|token| UUID { py: token, data: uuid::NAMESPACE_URL }).unwrap())?;
    m.add("NAMESPACE_X500",
          py.init(|token| UUID { py: token, data: uuid::NAMESPACE_X500 }).unwrap())?;

    m.add("RFC_4122",
          format!("{:?}", uuid::UuidVariant::RFC4122))?;
    m.add("RESERVED_NCS",
          format!("{:?}", uuid::UuidVariant::NCS))?;
    m.add("RESERVED_MICROSOFT",
          format!("{:?}", uuid::UuidVariant::Microsoft))?;
    m.add("RESERVED_FUTURE",
          format!("{:?}", uuid::UuidVariant::Future))?;

    Ok(())
}

pub fn register_classes(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<UUID>()?;
    Ok(())
}


////////////////////////////////////////
// CPython's entry point
////////////////////////////////////////

#[py::modinit(uuid_rpy)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    register_constants(py, m)?;
    register_classes(py, m)?;

    #[pyfn(m, "uuid3")]
    fn uuid3(py: Python, namespace: &UUID, name: &str)
          -> PyResult<Py<UUID>> {
        py.init(|token| {
            UUID {
                py: token,
                data: uuid::Uuid::new_v3(&namespace.data, name),
            }
        })
    }

    #[pyfn(m, "uuid4")]
    fn uuid4(py: Python) -> PyResult<Py<UUID>> {
        py.init(|token| {
            UUID {
                py: token,
                data: uuid::Uuid::new_v4(),
            }
        })
    }

    #[pyfn(m, "uuid5")]
    fn uuid5(py: Python, namespace: &UUID, name: &str)
          -> PyResult<Py<UUID>> {
        py.init(|token| {
            UUID {
                py: token,
                data: uuid::Uuid::new_v5(&namespace.data, name),
            }
        })
    }

    Ok(())
}
