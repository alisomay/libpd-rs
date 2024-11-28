use std::ffi::{CStr, CString};
use std::fmt::{self, Display};
use std::ptr;

use crate::error::{InstanceError, PdError, StringConversionError};

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

impl Atom {
    /// Converts a Rust `Atom` to a C `t_atom`.
    ///
    /// For symbols, this function requires a current libpd instance to be set.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`NoCurrentInstanceSet`](crate::error::InstanceError::NoCurrentInstanceSet)
    /// - [`StringConversion`](crate::error::PdError::StringConversion)
    pub fn to_t_atom(&self) -> Result<libpd_sys::t_atom, PdError> {
        match self {
            Self::Float(value) => {
                let mut t_atom = libpd_sys::t_atom {
                    a_type: libpd_sys::t_atomtype_A_FLOAT,
                    a_w: libpd_sys::word { w_float: 0.0 },
                };
                let p = &mut t_atom as *mut libpd_sys::t_atom;
                unsafe {
                    libpd_sys::libpd_set_double(p, *value);
                }
                Ok(t_atom)
            }
            Self::Symbol(s) => {
                if unsafe { libpd_sys::libpd_this_instance().is_null() } {
                    return Err(InstanceError::NoCurrentInstanceSet.into());
                }
                let c_str = CString::new(s.as_str()).map_err(StringConversionError::from)?;
                let sym_ptr = unsafe { libpd_sys::gensym(c_str.as_ptr()) };
                let t_atom = libpd_sys::t_atom {
                    a_type: libpd_sys::t_atomtype_A_SYMBOL,
                    a_w: libpd_sys::word { w_symbol: sym_ptr },
                };
                Ok(t_atom)
            }
        }
    }

    /// Converts a C `t_atom` to a Rust `Atom`.
    pub fn from_t_atom(t_atom: &libpd_sys::t_atom) -> Option<Self> {
        match t_atom.a_type {
            libpd_sys::t_atomtype_A_FLOAT => {
                let p = ptr::from_ref::<libpd_sys::t_atom>(t_atom).cast_mut();
                let value = unsafe { libpd_sys::libpd_get_double(p) };
                Some(Self::Float(value))
            }
            libpd_sys::t_atomtype_A_SYMBOL => {
                let p = ptr::from_ref::<libpd_sys::t_atom>(t_atom).cast_mut();
                let sym_ptr = unsafe { libpd_sys::libpd_get_symbol(p) };
                if sym_ptr.is_null() {
                    None
                } else {
                    // We trust the symbols we receive from the pd patch.
                    // If this proves that this assumption is not true this can panic.
                    let c_str = unsafe { CStr::from_ptr(sym_ptr) };
                    c_str.to_str().ok().map(|s| Self::Symbol(s.to_owned()))
                }
            }
            _ => None,
        }
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Float(value) => write!(f, "{value}"),
            Self::Symbol(s) => write!(f, "{s}"),
        }
    }
}

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

/// Convenience function to convert a list of `Atom`s to a list of `t_atom`s.
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`NoCurrentInstanceSet`](crate::error::InstanceError::NoCurrentInstanceSet)
/// - [`StringConversion`](crate::error::PdError::StringConversion)
pub fn make_t_atom_list_from_atom_list(atoms: &[Atom]) -> Result<Vec<libpd_sys::t_atom>, PdError> {
    atoms.iter().map(Atom::to_t_atom).collect()
}

/// Convenience function to convert a list of `t_atom`s to a list of `Atom`s.
///
/// This function will ignore any `t_atom`s that cannot be converted to `Atom`.
pub fn make_atom_list_from_t_atom_list(t_atoms: &[libpd_sys::t_atom]) -> Vec<Atom> {
    t_atoms.iter().filter_map(Atom::from_t_atom).collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::restriction, clippy::nursery, clippy::all, clippy::pedantic)]
    use crate::instance::PdInstance;

    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_float_conversion() {
        // Test converting a float Atom to t_atom and back
        let original_atom = Atom::Float(3.14);
        let t_atom = original_atom
            .to_t_atom()
            .expect("Conversion to t_atom failed");
        let converted_atom = Atom::from_t_atom(&t_atom).expect("Conversion from t_atom failed");
        assert_eq!(original_atom, converted_atom);
    }

    #[test]
    #[serial]
    fn test_symbol_conversion() {
        // Initialize libpd before using symbols
        let main_instance = PdInstance::new().expect("Failed to create Pd instance");
        main_instance.set_as_current();

        // Test converting a symbol Atom to t_atom and back
        let original_atom = Atom::Symbol("test_symbol".to_string());
        let t_atom = original_atom
            .to_t_atom()
            .expect("Conversion to t_atom failed");
        let converted_atom = Atom::from_t_atom(&t_atom).expect("Conversion from t_atom failed");
        assert_eq!(original_atom, converted_atom);
    }

    #[test]
    #[serial]
    fn test_atom_list_conversion() {
        // Initialize libpd before using symbols
        let main_instance = PdInstance::new().expect("Failed to create Pd instance");
        main_instance.set_as_current();

        // Create a list of Atoms
        let original_atoms = vec![
            Atom::Float(1.23),
            Atom::Symbol("hello".to_string()),
            Atom::Float(4.56),
            Atom::Symbol("world".to_string()),
        ];

        // Convert the list of Atoms to a list of t_atoms
        let t_atoms = make_t_atom_list_from_atom_list(&original_atoms)
            .expect("Conversion to t_atom list failed");

        // Convert the list of t_atoms back to a list of Atoms
        let converted_atoms = make_atom_list_from_t_atom_list(&t_atoms);

        // Assert that the original and converted lists are equal
        assert_eq!(original_atoms, converted_atoms);
    }

    #[test]
    #[serial]
    fn test_empty_atom_list_conversion() {
        // Test converting an empty list of Atoms
        let original_atoms: Vec<Atom> = Vec::new();
        let t_atoms = make_t_atom_list_from_atom_list(&original_atoms)
            .expect("Conversion to t_atom list failed");
        let converted_atoms = make_atom_list_from_t_atom_list(&t_atoms);
        assert_eq!(original_atoms, converted_atoms);
    }

    #[test]
    #[serial]
    fn test_float_reference_conversion() {
        // Test converting a reference to a float to Atom
        let value = 2.718;
        let atom = Atom::from(&value);
        assert_eq!(atom, Atom::Float(2.718));

        // Convert to t_atom and back
        let t_atom = atom.to_t_atom().expect("Conversion to t_atom failed");
        let converted_atom = Atom::from_t_atom(&t_atom).expect("Conversion from t_atom failed");
        assert_eq!(atom, converted_atom);
    }

    #[test]
    #[serial]
    fn test_symbol_reference_conversion() {
        // Initialize libpd before using symbols
        let main_instance = PdInstance::new().expect("Failed to create Pd instance");
        main_instance.set_as_current();

        // Test converting a &str to Atom
        let value = "reference_symbol";
        let atom = Atom::from(value);
        assert_eq!(atom, Atom::Symbol("reference_symbol".to_string()));

        // Convert to t_atom and back
        let t_atom = atom.to_t_atom().expect("Conversion to t_atom failed");
        let converted_atom = Atom::from_t_atom(&t_atom).expect("Conversion from t_atom failed");
        assert_eq!(atom, converted_atom);
    }

    #[test]
    #[serial]
    fn test_char_conversion() {
        // Initialize libpd before using symbols
        let main_instance = PdInstance::new().expect("Failed to create Pd instance");
        main_instance.set_as_current();

        // Test converting a char to Atom
        let value = 'a';
        let atom = Atom::from(value);
        assert_eq!(atom, Atom::Symbol("a".to_string()));

        // Convert to t_atom and back
        let t_atom = atom.to_t_atom().expect("Conversion to t_atom failed");
        let converted_atom = Atom::from_t_atom(&t_atom).expect("Conversion from t_atom failed");
        assert_eq!(atom, converted_atom);
    }

    #[test]
    #[serial]
    fn test_number_type_conversions() {
        // Test various number types
        let int_value: i32 = -42;
        let atom = Atom::from(int_value);
        assert_eq!(atom, Atom::Float(-42.0));

        let uint_value: u32 = 42;
        let atom = Atom::from(uint_value);
        assert_eq!(atom, Atom::Float(42.0));

        let float_value: f32 = 3.14;
        let atom = Atom::from(float_value);
        if let Atom::Float(value) = atom {
            assert!(
                (value - 3.14).abs() < 1e-6,
                "Expected value close to 3.14, got {}",
                value
            );
        } else {
            panic!("Expected Atom::Float");
        }

        let double_value: f64 = 2.71828;
        let atom = Atom::from(double_value);
        if let Atom::Float(value) = atom {
            assert!(
                (value - 2.71828).abs() < 1e-10,
                "Expected value close to 2.71828, got {}",
                value
            );
        } else {
            panic!("Expected Atom::Float");
        }
    }

    #[test]
    #[serial]
    fn test_symbol_with_null_byte() {
        // Initialize libpd before using symbols
        let main_instance = PdInstance::new().expect("Failed to create Pd instance");
        main_instance.set_as_current();

        // Test creating a symbol with a null byte (should fail)
        let symbol_with_null = "null\0byte";
        let atom_result = Atom::from(symbol_with_null).to_t_atom();

        assert!(atom_result.is_err());
    }

    #[test]
    #[serial]
    fn test_large_float_conversion() {
        let value = 1e308; // Large f64 value
        let atom = Atom::Float(value);
        let t_atom = atom.to_t_atom().expect("Conversion to t_atom failed");
        let converted_atom = Atom::from_t_atom(&t_atom).expect("Conversion from t_atom failed");
        assert_eq!(atom, converted_atom);
    }

    #[test]
    #[serial]
    fn test_unicode_symbol_conversion() {
        let main_instance = PdInstance::new().expect("Failed to create Pd instance");
        main_instance.set_as_current();

        let symbol = "こんにちは"; // "Hello" in Japanese
        let atom = Atom::Symbol(symbol.to_string());
        let t_atom = atom.to_t_atom().expect("Conversion to t_atom failed");
        let converted_atom = Atom::from_t_atom(&t_atom).expect("Conversion from t_atom failed");
        assert_eq!(atom, converted_atom);
    }

    #[test]
    #[serial]
    fn test_empty_symbol_conversion() {
        let main_instance = PdInstance::new().expect("Failed to create Pd instance");
        main_instance.set_as_current();

        let atom = Atom::Symbol(String::new());
        let t_atom = atom.to_t_atom().expect("Conversion to t_atom failed");
        let converted_atom = Atom::from_t_atom(&t_atom).expect("Conversion from t_atom failed");
        assert_eq!(atom, converted_atom);
    }
}
