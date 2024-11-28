use core::ffi;
use std::fmt::{self, Display};

/// A type to represent a pd Atom type in Rust side.
///
/// Pd has floating point numbers and symbols as primitive types.
/// This enum maps those to their Rust counterparts.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Atom {
    /// A floating point number from pd.
    Float(f64),
    /// A symbol from pd. Symbols are interned in pd, but it can be treated as Strings in Rust.
    Symbol(String),
}

macro_rules! atom_from_number_type {
    ($type:ty) => {
        impl From<$type> for Atom {
            fn from(value: $type) -> Self {
                Self::Float(value.into())
            }
        }
    };
}

macro_rules! atom_from_reference_number_type {
    ($type:ty) => {
        impl From<$type> for Atom {
            fn from(value: $type) -> Self {
                Self::Float((*value).into())
            }
        }
    };
}

atom_from_number_type!(i8);
atom_from_number_type!(i16);
atom_from_number_type!(i32);
atom_from_number_type!(u8);
atom_from_number_type!(u16);
atom_from_number_type!(u32);
atom_from_number_type!(f32);
atom_from_number_type!(f64);
atom_from_reference_number_type!(&i8);
atom_from_reference_number_type!(&i16);
atom_from_reference_number_type!(&i32);
atom_from_reference_number_type!(&u8);
atom_from_reference_number_type!(&u16);
atom_from_reference_number_type!(&u32);
atom_from_reference_number_type!(&f32);
atom_from_reference_number_type!(&f64);

impl From<String> for Atom {
    fn from(s: String) -> Self {
        Self::Symbol(s)
    }
}
impl From<&String> for Atom {
    fn from(s: &String) -> Self {
        Self::Symbol(s.clone())
    }
}
impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Self::Symbol(s.to_owned())
    }
}
impl From<char> for Atom {
    fn from(c: char) -> Self {
        Self::Symbol(c.to_string())
    }
}
impl From<&char> for Atom {
    fn from(c: &char) -> Self {
        Self::Symbol(c.to_string())
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Float(float) => write!(f, "{float}"),
            Self::Symbol(s) => write!(f, "{s}"),
        }
    }
}

/// The handle which is returned from opening a patch.
///
/// This is a [`c_void`](std::ffi::c_void) in the underlying sys crate but for convenience it is converted to `usize` and held here.
///
/// This handle should be kept alive for the lifetime of the patch.
#[derive(Debug)]
pub struct PatchFileHandle(*mut ffi::c_void);
impl PatchFileHandle {
    pub(crate) const fn as_mut_ptr(&self) -> *mut ffi::c_void {
        self.0
    }

    pub const fn into_inner(self) -> *mut ffi::c_void {
        self.0
    }
}

impl From<*mut ffi::c_void> for PatchFileHandle {
    fn from(ptr: *mut ffi::c_void) -> Self {
        Self(ptr)
    }
}

/// The handle which is returned from subscribing to a sender.
///
/// This is a [`c_void`](std::ffi::c_void) in the underlying sys crate but for convenience it is converted to `usize` and held here.
///
/// This handle could be used later to unsubscribe from the sender.
#[derive(Debug)]
pub struct ReceiverHandle(*mut ffi::c_void);

impl ReceiverHandle {
    pub const fn into_inner(self) -> *mut ffi::c_void {
        self.0
    }
}

impl From<*mut ffi::c_void> for ReceiverHandle {
    fn from(ptr: *mut ffi::c_void) -> Self {
        Self(ptr)
    }
}

// pub const t_atomtype_A_SEMI: t_atomtype = 4;
// pub const t_atomtype_A_COMMA: t_atomtype = 5;
// pub const t_atomtype_A_DOLLAR: t_atomtype = 8;
// pub const t_atomtype_A_DOLLSYM: t_atomtype = 9;

// #define SETDOLLAR(atom, n) ((atom)->a_type = A_DOLLAR, \
//     (atom)->a_w.w_index = (n))
// #define SETDOLLSYM(atom, s) ((atom)->a_type = A_DOLLSYM, \
//     (atom)->a_w.w_symbol= (s))
// #define SETSEMI(atom) ((atom)->a_type = A_SEMI, (atom)->a_w.w_index = 0)
// #define SETCOMMA(atom) ((atom)->a_type = A_COMMA, (atom)->a_w.w_index = 0)

// Appendix, types related to atoms.
//
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

// #define SETSEMI(atom) ((atom)->a_type = A_SEMI, (atom)->a_w.w_index = 0)
// #define SETCOMMA(atom) ((atom)->a_type = A_COMMA, (atom)->a_w.w_index = 0)
// #define SETPOINTER(atom, gp) ((atom)->a_type = A_POINTER, \
//     (atom)->a_w.w_gpointer = (gp))
// #define SETFLOAT(atom, f) ((atom)->a_type = A_FLOAT, (atom)->a_w.w_float = (f))
// #define SETSYMBOL(atom, s) ((atom)->a_type = A_SYMBOL, \
//     (atom)->a_w.w_symbol = (s))
// #define SETDOLLAR(atom, n) ((atom)->a_type = A_DOLLAR, \
//     (atom)->a_w.w_index = (n))
// #define SETDOLLSYM(atom, s) ((atom)->a_type = A_DOLLSYM, \
//     (atom)->a_w.w_symbol= (s))

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
