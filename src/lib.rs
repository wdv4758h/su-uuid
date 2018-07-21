//! Python "uuid" module in Rust, as CPython extension.
//! Wrapping the Rust "uuid" crate up to Python with PyO3.

#![feature(proc_macro)]     // for "pyfn" "pyfunction" "pymodinit" "pyproto" ...
#![feature(specialization)] // for "pymethods" "pyclass"
#![feature(concat_idents)]  // for "wrap_function!"
#![allow(non_snake_case)]   // don't show warning for "wrap_function!" related stuff


extern crate pyo3;
extern crate uuid;
extern crate arrayvec;
#[macro_use]
extern crate lazy_static;


use pyo3::prelude::*;
use pyo3::{pymodinit, pyproto, pyclass, pymethods, pyfunction};
use pyo3::wrap_function;
use pyo3::exc;
use std::str::FromStr;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::io;
use std::io::prelude::*;
use std::ops::BitAnd;
use std::ops::BitOr;
use arrayvec::ArrayVec;


/// Fake SafeUUID implementation
#[pyclass]
struct SafeUUID {}

#[pymethods]
impl SafeUUID {
    #[getter]
    fn safe(&self) -> PyResult<u8> {
        Ok(0)
    }

    // FIXME: _unsafe -> unsafe
    #[getter]
    fn _unsafe(&self) -> PyResult<i8> {
        Ok(-1)
    }

    #[getter]
    fn unknown(&self) -> PyResult<Option<u8>> {
        Ok(None)
    }
}


fn get_mac_addresses() -> io::Result<Vec<String>> {
    use std::fs;
    use std::path::Path;
    let dir = Path::new("/sys/class/net");
    if dir.is_dir() {
        let mut addresses = vec![];
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let address_file = entry.path().join("address");
            if address_file.exists() {
                let mut contents = String::new();
                fs::File::open(address_file)?.read_to_string(&mut contents)?;
                contents.truncate(17);
                addresses.push(contents);
            }
        }
        Ok(addresses)
    } else {
        Ok(vec![])
    }
}

fn get_node() -> ArrayVec<[u8; 6]> {
    let addresses = get_mac_addresses().unwrap();
    assert!(addresses.len() > 0);
    addresses.iter().filter(|s| !s.contains("00:00:00:00:00:00")).next().unwrap()
             .split(':').map(|s| u8::from_str_radix(s, 16).unwrap()).collect()
}

fn clean_uuid_string(string: &str) -> String {
    // FIXME: probably use "trim_matches" here
    let patterns: &[_] = &['{', '}'];
    string.replace("urn:", "")
          .replace("uuid:", "")
          .trim_left_matches(patterns)
          .trim_right_matches(patterns)
          .replace("-", "")
}


#[pyclass]
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
        ret
    }

    fn get_time(&self) -> u128 {
        (((self.data.as_fields().2 as u128 & 0x0fff) << 48) |
         ((self.data.as_fields().1 as u128) << 32) |
         self.data.as_fields().0 as u128)
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.data.hash(&mut hasher);
        hasher.finish()
    }
}


/// create the UUID instance
///
/// UUID('{12345678-1234-5678-1234-567812345678}')
/// UUID('12345678123456781234567812345678')
/// UUID('urn:uuid:12345678-1234-5678-1234-567812345678')
/// UUID(bytes='\x12\x34\x56\x78'*4)
/// UUID(bytes_le='\x78\x56\x34\x12\x34\x12\x78\x56' +
///               '\x12\x34\x56\x78\x12\x34\x56\x78')
/// UUID(fields=(0x12345678, 0x1234, 0x5678, 0x12, 0x34, 0x567812345678))
/// UUID(int=0x12345678123456781234567812345678)
///
#[pymethods]
impl UUID {
    #[new]
    #[args(hex="None", bytes="None", bytes_le="None", args="*")]
    fn __new__(obj: &PyRawObject,
               hex: Option<Option<&str>>,
               // FIXME: use [u8; 16]
               bytes: Option<Option<Vec<u8>>>,    // FIXME: use reference directly
               // FIXME: use [u8; 16]
               bytes_le: Option<Option<Vec<u8>>>, // FIXME: use reference directly
               fields: Option<(u32, u16, u16, u8, u8, u64)>,
               int: Option<u128>,
               version: Option<u8>)
      -> PyResult<()> {

        // FIXME: remove this nested "Option"
        let hex = hex.unwrap();
        let bytes = bytes.unwrap();
        let bytes_le = bytes_le.unwrap();

        // check the arguments count, we shouldn't have multiple of them
        let args_count = [hex.is_some(),
                          bytes.is_some(),
                          bytes_le.is_some(),
                          fields.is_some(),
                          int.is_some()].iter().filter(|i| **i).count();
        if (args_count > 1) || (args_count == 0) {
            return Err(exc::TypeError::new("passing wrong number of arguments"));
        }

        let uuid =
            // check the "hex"
            if let Some(hex) = hex {
                let string = clean_uuid_string(hex);
                if string.len() != 32 {
                    return Err(exc::ValueError::new("badly formed hexadecimal UUID string"));
                }
                uuid::Uuid::from_str(&string)
            // check the "bytes_le"
            } else if let Some(bytes_le) = bytes_le {
                if bytes_le.len() != 16 {
                    return Err(exc::ValueError::new("bytes_le is not a 16-char string"));
                }
                // FIXME: do not create vector
                let slice = bytes_le[..4].iter().rev()
                    .chain(bytes_le[4..6].iter().rev())
                    .chain(bytes_le[6..8].iter().rev())
                    .chain(bytes_le[8..].iter())
                    .map(|n| *n);
                uuid::Uuid::from_bytes(
                    slice.collect::<Vec<_>>().as_slice())
            // check the "bytes"
            } else if let Some(bytes) = bytes {
                if bytes.len() != 16 {
                    return Err(exc::ValueError::new("bytes is not a 16-char string"));
                }
                uuid::Uuid::from_bytes(&bytes)
            // check the "fields"
            } else if let Some(fields) = fields {
                // node field actually only have 48 bits
                if fields.5 >= (1 << 48) {
                    return Err(exc::ValueError::new("field 6 out of range (need a 48-bit value)"));
                }

                let ptr = (&fields.5) as *const u64 as *const [u8; 8];
                // this is safe, we have exactly that size of data
                let node = unsafe { *ptr };

                uuid::Uuid::from_fields(
                    fields.0,
                    fields.1,
                    fields.2,
                    &[fields.3, fields.4,
                      node[5], node[4],
                      node[3], node[2],
                      node[1], node[0]])
            // check the "int"
            } else if let Some(int) = int {
                Ok(uuid::Uuid::from_u128(int.swap_bytes()))
            } else {
                // we shouldn't go here
                unreachable!()
            };

        // check "version"
        if let Some(version) = version {
            if (version < 1) || (version > 5) {
                return Err(exc::ValueError::new("illegal version number"));
            }
            // FIXME: set the version into UUID instance
        }

        // FIXME: set the "is_safe" attribute

        let uuid = match uuid {
            Ok(uuid) => uuid,
            Err(e) => return Err(exc::ValueError::new(format!("UUID creation has error: {}", e))),
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
    pub fn clock_seq(&self) -> PyResult<u16> {
        let hi = self.data.as_fields().3[0] as u16;
        let low = self.data.as_fields().3[1] as u16;
        Ok(hi.bitand(0x3f)
             .wrapping_shl(8)
             .bitor(low))
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
    pub fn int(&self) -> PyResult<u128> {
        Ok(self.get_u128())
    }

    #[getter]
    pub fn node(&self) -> PyResult<u64> {
        // FIXME: more efficient way ?
        Ok(self.data.as_fields().3[2..]
               .iter().fold(0_u64, |a, &b| { a*256+(b as u64) }))
    }

    #[getter]
    pub fn time(&self) -> PyResult<u128> {
        // 60 bits timestamp
        Ok(self.get_time())
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
    pub fn version(&self) -> PyResult<PyObject> {
        match self.data.get_variant() {
            // the version bits are only meaningful for RFC 4122 UUIDs
            Some(uuid::UuidVariant::RFC4122) => {
                Ok(self.data.get_version_num().to_object(self.py()))
            },
            _ => Ok(self.py().None()),
        }
    }

    #[getter]
    pub fn is_safe(&self) -> PyResult<u8> {
        let tmp = SafeUUID {};
        tmp.safe()
    }
}

#[pyproto]
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
            CompareOp::Lt => Ok(self.data <  other.data),
            CompareOp::Le => Ok(self.data <= other.data),
            CompareOp::Gt => Ok(self.data >  other.data),
            CompareOp::Ge => Ok(self.data >= other.data),
        }
    }

    fn __hash__(&self) -> PyResult<isize> {
        Ok(self.hash() as isize)
    }
}


#[pyproto]
impl PyNumberProtocol for UUID {
    fn __int__(&self) -> PyResult<u128> {
        self.int()
    }
}


#[pyfunction]
fn _load_system_functions() -> Option<bool> {
    None
}

#[pyfunction]
fn _generate_time_safe() -> Option<bool> {
    None
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

    m.add("_has_uuid_generate_time_safe", true)?;
    m.add_function(wrap_function!(_load_system_functions))?;
    m.add_function(wrap_function!(_generate_time_safe))?;

    Ok(())
}

pub fn register_classes(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<UUID>()?;
    m.add_class::<SafeUUID>()?;
    Ok(())
}


////////////////////////////////////////
// CPython's entry point
////////////////////////////////////////

#[pymodinit]
fn su_uuid(py: Python, m: &PyModule) -> PyResult<()> {
    register_constants(py, m)?;
    register_classes(py, m)?;

    #[pyfn(m, "getnode")]
    fn getnode(_py: Python) -> PyResult<u64> {
        let node = get_node();
        Ok(node.iter().fold(0_u64, |a, &b| { a*256+(b as u64) }))
    }

    #[pyfn(m, "uuid1", args="*")]
    fn uuid1(py: Python,
             node: Option<u64>,
             clock_seq: Option<u16>,
             _args: Option<&PyTuple>)
          -> PyResult<Py<UUID>> {

        use std::time::{SystemTime, UNIX_EPOCH};

        let clock_seq = if let Some(clock_seq) = clock_seq {
            clock_seq
        } else {
            // FIXME: generate random number
            0
        };

        let now = SystemTime::now();
        let dur = now.duration_since(UNIX_EPOCH).unwrap();
        let ctx = uuid::UuidV1Context::new(clock_seq);

        let node: ArrayVec<[u8; 6]> =
            if node.is_some() {
                let mut value = node.unwrap();

                let ptr = (&value) as *const u64 as *const [u8; 8];
                // this is safe, we have exactly that size of data
                let tmp = unsafe { *ptr };
                tmp.iter().rev().skip(2).map(|x| *x).collect()

            } else {
                lazy_static! {
                  static ref NODE: ArrayVec<[u8; 6]> = get_node();
                }
                NODE.clone()
            };

        py.init(|token| {
            UUID {
                py: token,
                data: uuid::Uuid::new_v1(&ctx,
                                         dur.as_secs(),
                                         dur.subsec_nanos(),
                                         &node).unwrap(),
            }
        })
    }

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
