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

/// A type to represent a pd Atom type in Rust side.
///
/// Pd has floating point numbers and symbols as primitive types.
/// This enum maps those to their Rust counterparts.
#[derive(Debug, Clone)]
pub enum Atom {
    Float(f32),
    Symbol(String),
    Double(f64),
}

impl From<f32> for Atom {
    fn from(f: f32) -> Self {
        Self::Float(f)
    }
}
impl From<f64> for Atom {
    fn from(d: f64) -> Self {
        Self::Double(d)
    }
}
impl From<String> for Atom {
    fn from(s: String) -> Self {
        Self::Symbol(s)
    }
}
impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Self::Symbol(s.to_owned())
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

/// The handle which is returned from opening a patch.
///
/// This is a `std::ffi::c_void` in the underlying sys crate but for convenience it is converted to `usize` and held here.
///
/// This handle should be kept alive for the lifetime of the patch.
#[derive(Debug)]
pub struct PatchFileHandle(usize);
impl PatchFileHandle {
    pub(crate) const fn as_mut_ptr(&self) -> *mut std::ffi::c_void {
        self.0 as *mut std::ffi::c_void
    }
}

impl From<*mut std::ffi::c_void> for PatchFileHandle {
    fn from(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr as usize)
    }
}

// Needed in this case.
#[allow(clippy::from_over_into)]
impl Into<*mut std::ffi::c_void> for PatchFileHandle {
    fn into(self) -> *mut std::ffi::c_void {
        self.0 as *mut std::ffi::c_void
    }
}

/// The handle which is returned from subscribing to a sender.
///
/// This is a `std::ffi::c_void` in the underlying sys crate but for convenience it is converted to `usize` and held here.
///
/// This handle could be used later to unsubscribe from the sender.
#[derive(Debug)]
pub struct ReceiverHandle(usize);

impl From<*mut std::ffi::c_void> for ReceiverHandle {
    fn from(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr as usize)
    }
}

// Might be needed
#[allow(clippy::from_over_into)]
impl Into<*mut std::ffi::c_void> for ReceiverHandle {
    fn into(self) -> *mut std::ffi::c_void {
        self.0 as *mut std::ffi::c_void
    }
}
