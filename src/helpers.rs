#![allow(clippy::redundant_pub_crate)]

/// Transforms an iterable of type `Atom` to a `Vec<libpd_sys::t_atom>`.
macro_rules! make_t_atom_list_from_atom_list {
    ($list: expr) => {
        $list
            .into_iter()
            .map(|atom_variant| match atom_variant {
                Atom::Float(value) => {
                    let t_atom = libpd_sys::t_atom {
                        a_type: libpd_sys::t_atomtype_A_FLOAT,
                        a_w: libpd_sys::word { w_float: *value },
                    };
                    let p = std::ptr::from_ref::<libpd_sys::t_atom>(&t_atom).cast_mut();
                    // Using a setter us crucial or else float values become 0s when sending a list.
                    unsafe {
                        libpd_sys::libpd_set_double(p, *value);
                    }
                    t_atom
                }

                // If there will be a bug related to this later,
                // Try using libpd_sys::libpd_set_symbol instead of manually setting the value.
                Atom::Symbol(value) => libpd_sys::t_atom {
                    a_type: libpd_sys::t_atomtype_A_SYMBOL,
                    a_w: libpd_sys::word {
                        w_symbol: unsafe {
                            let sym = CString::new(value.to_owned()).unwrap();
                            libpd_sys::gensym(sym.as_ptr())
                        },
                    },
                },
                // TODO: See if there are more cases to be covered.
            })
            .collect::<Vec<libpd_sys::t_atom>>()
    };
}

/// Transforms an iterable of type `t_atom` to a `Vec<Atom>`.
macro_rules! make_atom_list_from_t_atom_list {
    ($list: expr) => {
        $list
            .into_iter()
            .map(|atom_type| match atom_type.a_type {
                libpd_sys::t_atomtype_A_FLOAT => {
                    let ptr_to_inner =
                        std::ptr::from_ref::<libpd_sys::t_atom>(atom_type).cast_mut();
                    let f: f64 = unsafe { libpd_sys::libpd_get_double(ptr_to_inner) };
                    Atom::Float(f)
                }
                libpd_sys::t_atomtype_A_SYMBOL => {
                    let ptr_to_inner =
                        std::ptr::from_ref::<libpd_sys::t_atom>(atom_type).cast_mut();
                    let sym: *const std::os::raw::c_char =
                        unsafe { libpd_sys::libpd_get_symbol(ptr_to_inner) };
                    let result = unsafe { CStr::from_ptr(sym) };
                    Atom::Symbol(result.to_str().unwrap().to_owned())
                }
                // TODO: See if there are more cases to be covered.
                _ => unimplemented!(),
            })
            .collect::<Vec<Atom>>()
    };
}

pub(crate) use {make_atom_list_from_t_atom_list, make_t_atom_list_from_atom_list};
