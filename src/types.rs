use std::ffi::{CStr, CString};

// TODO: Is really implementing all of these needed?
// pub type t_word = word;
// pub const t_atomtype_A_NULL: t_atomtype = 0;
// pub const t_atomtype_A_FLOAT: t_atomtype = 1;
// pub const t_atomtype_A_SYMBOL: t_atomtype = 2;
// pub const t_atomtype_A_POINTER: t_atomtype = 3;
// pub const t_atomtype_A_SEMI: t_atomtype = 4;
// pub const t_atomtype_A_COMMA: t_atomtype = 5;
// pub const t_atomtype_A_DEFFLOAT: t_atomtype = 6;
// pub const t_atomtype_A_DEFSYM: t_atomtype = 7;
// pub const t_atomtype_A_DOLLAR: t_atomtype = 8;
// pub const t_atomtype_A_DOLLSYM: t_atomtype = 9;
// pub const t_atomtype_A_GIMME: t_atomtype = 10;
// pub const t_atomtype_A_CANT: t_atomtype = 11;
// pub type t_atomtype = ::std::os::raw::c_uint;

// #[repr(C)]
// #[derive(Copy, Clone)]
// pub struct _atom {
//     pub a_type: t_atomtype,
//     pub a_w: word,
// }

// pub union word {
//     pub w_float: t_float,
//     pub w_symbol: *mut t_symbol,
//     pub w_gpointer: *mut t_gpointer,
//     pub w_array: *mut _array,
//     pub w_binbuf: *mut _binbuf,
//     pub w_index: ::std::os::raw::c_int,
// }

#[derive(Debug, Clone)]
pub enum Atom {
    Float(f32),
    Symbol(String),
    Double(f64),
}

impl From<f32> for Atom {
    fn from(f: f32) -> Self {
        Atom::Float(f)
    }
}
impl From<f64> for Atom {
    fn from(d: f64) -> Self {
        Atom::Double(d)
    }
}
impl From<String> for Atom {
    fn from(s: String) -> Self {
        Atom::Symbol(s)
    }
}
impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Atom::Symbol(s.to_owned())
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Atom::Float(float) => write!(f, "{}", float),
            Atom::Symbol(s) => write!(f, "{}", s),
            Atom::Double(d) => write!(f, "{}", d),
            // PdType::Pointer => write!(f, "Pointer"),
            // PdType::Array => write!(f, "Array"),
            // PdType::Binbuf => write!(f, "Binbuf"),
            // PdType::Index => write!(f, "Index"),
        }
    }
}

impl Atom {
    // pub(crate) fn as_mut_ptr(&self) -> *mut libpd_sys::t_atom {
    //     match self {
    //         Atom::Float(f) => {}
    //         Atom::Symbol(s) => {}
    //         Atom::Double(d) => {}
    //     }
    // }
    // pub fn get_value(&self) -> PdType {
    //     match self.inner.a_type {
    //         libpd_sys::t_atomtype_A_FLOAT => {
    //             let ptr_to_inner =
    //                 &self.inner as *const libpd_sys::t_atom as *mut libpd_sys::t_atom;
    //             let f: f32 = unsafe { libpd_sys::libpd_get_float(ptr_to_inner) };
    //             PdType::Float(f)
    //         }
    //         libpd_sys::t_atomtype_A_SYMBOL => {
    //             let ptr_to_inner =
    //                 &self.inner as *const libpd_sys::t_atom as *mut libpd_sys::t_atom;
    //             let sym: *const std::os::raw::c_char =
    //                 unsafe { libpd_sys::libpd_get_symbol(ptr_to_inner) };
    //             let result = unsafe { CStr::from_ptr(sym) };
    //             PdType::Symbol(result.to_str().unwrap().to_owned())
    //         }
    //         _ => todo!(),
    //     }
    // }

    // pub fn set_value(&mut self, value: PdType) {
    //     match value {
    //         PdType::Float(val) => self.set_float(val),
    //         PdType::Symbol(sym) => self.set_symbol(sym),
    //         _ => todo!(),
    //     }
    // }

    // /// write a float value to the given atom
    // fn set_float(&mut self, value: f32) {
    //     unsafe {
    //         libpd_sys::libpd_set_float(
    //             &self.inner as *const libpd_sys::t_atom as *mut libpd_sys::t_atom,
    //             value,
    //         );
    //     }
    // }

    // /// write a symbol value to the given atom
    // fn set_symbol<T: AsRef<str>>(&mut self, value: T) {
    //     let val = CString::new(value.as_ref()).unwrap();
    //     unsafe {
    //         libpd_sys::libpd_set_symbol(
    //             &self.inner as *const libpd_sys::t_atom as *mut libpd_sys::t_atom,
    //             val.as_ptr(),
    //         );
    //     }
    // }

    // TODO: Maybe implement get float and get symbol separately also with result types?
}

// impl From<libpd_sys::t_atom> for Atom {
//     fn from(atom: libpd_sys::t_atom) -> Self {
//         Atom { inner: atom }
//     }
// }

// impl From<f32> for Atom {
//     fn from(value: f32) -> Self {
//         Self {
//             inner: libpd_sys::t_atom {
//                 a_type: libpd_sys::t_atomtype_A_FLOAT,
//                 a_w: libpd_sys::word { w_float: value },
//             },
//         }
//     }
// }

// impl From<String> for Atom {
//     fn from(value: String) -> Self {
//         let val = CString::new(&*value).unwrap();
//         Self {
//             inner: libpd_sys::t_atom {
//                 a_type: libpd_sys::t_atomtype_A_SYMBOL,
//                 a_w: libpd_sys::word {
//                     w_symbol: unsafe { libpd_sys::gensym(val.as_ptr()) },
//                 },
//             },
//         }
//     }
// }

// impl From<&String> for Atom {
//     fn from(value: &String) -> Self {
//         let val = CString::new(&**value).unwrap();
//         Self {
//             inner: libpd_sys::t_atom {
//                 a_type: libpd_sys::t_atomtype_A_SYMBOL,
//                 a_w: libpd_sys::word {
//                     w_symbol: unsafe { libpd_sys::gensym(val.as_ptr()) },
//                 },
//             },
//         }
//     }
// }

// impl From<&str> for Atom {
//     fn from(value: &str) -> Self {
//         let val = CString::new(value).unwrap();
//         Self {
//             inner: libpd_sys::t_atom {
//                 a_type: libpd_sys::t_atomtype_A_SYMBOL,
//                 a_w: libpd_sys::word {
//                     w_symbol: unsafe { libpd_sys::gensym(val.as_ptr()) },
//                 },
//             },
//         }
//     }
// }

// TODO: The thing is one needs to find a way to either
// cast this to a type which is equivalent memory wise or do the same thing :)
// I think this would worth a research for this to be a clean library.
// TODO: This type has to be a smart pointer, we can not pass voids around.
// pub struct PatchFileHandle {
//     inner: *mut std::os::raw::c_void,
// }

// impl PatchFileHandle {
//     pub fn into_inner(self) -> *mut std::os::raw::c_void {
//         self.inner
//     }
// }
// impl From<*mut std::os::raw::c_void> for PatchFileHandle {
//     fn from(inner: *mut std::os::raw::c_void) -> Self {
//         Self { inner }
//     }
// }
pub struct PatchFileHandle(usize);
impl PatchFileHandle {
    pub(crate) fn as_mut_ptr(&self) -> *mut std::os::raw::c_void {
        self.0 as *mut std::os::raw::c_void
    }
}

impl From<*mut std::os::raw::c_void> for PatchFileHandle {
    fn from(ptr: *mut std::os::raw::c_void) -> Self {
        Self(ptr as usize)
    }
}

// Needed in this case.
#[allow(clippy::from_over_into)]
impl Into<*mut std::os::raw::c_void> for PatchFileHandle {
    fn into(self) -> *mut std::os::raw::c_void {
        self.0 as *mut std::os::raw::c_void
    }
}

pub struct ReceiverHandle {
    inner: *mut std::os::raw::c_void,
}

impl ReceiverHandle {
    pub fn into_inner(self) -> *mut std::os::raw::c_void {
        self.inner
    }
}
impl From<*mut std::os::raw::c_void> for ReceiverHandle {
    fn from(inner: *mut std::os::raw::c_void) -> Self {
        Self { inner }
    }
}
