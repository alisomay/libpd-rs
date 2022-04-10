#![allow(clippy::redundant_pub_crate)]
// TODO: Finish covering cases for these macros.
macro_rules! make_t_atom_list_from_atom_list {
    ($list: expr) => {
        $list
            .into_iter()
            .map(|atom_variant| {
                match atom_variant {
                    // TODO:
                    // There is only float in the atom type.
                    // Probably precision changes depending on pd compilation float size.
                    // Cover for that here.
                    Atom::Float(value) => libpd_sys::t_atom {
                        a_type: libpd_sys::t_atomtype_A_FLOAT,
                        a_w: libpd_sys::word { w_float: *value },
                    },
                    Atom::Symbol(value) => libpd_sys::t_atom {
                        a_type: libpd_sys::t_atomtype_A_SYMBOL,
                        a_w: libpd_sys::word {
                            w_symbol: libpd_sys::gensym(
                                CString::new(value.to_owned()).unwrap().as_ptr(),
                            ),
                        },
                    },
                    Atom::Double(value) => {
                        libpd_sys::t_atom {
                            a_type: libpd_sys::t_atomtype_A_FLOAT,
                            a_w: libpd_sys::word {
                                // TODO: This is wrong.
                                // Please find a better strategy here.
                                w_float: *value as f32,
                            },
                        }
                    }
                }
            })
            .collect::<Vec<libpd_sys::t_atom>>()
    };
}

macro_rules! make_atom_list_from_t_atom_list {
    ($list: expr) => {
        $list
            .into_iter()
            .map(|atom_type| match atom_type.a_type {
                libpd_sys::t_atomtype_A_FLOAT => {
                    let ptr_to_inner =
                        atom_type as *const libpd_sys::t_atom as *mut libpd_sys::t_atom;
                    let f: f32 = unsafe { libpd_sys::libpd_get_float(ptr_to_inner) };
                    Atom::Float(f)
                }
                libpd_sys::t_atomtype_A_SYMBOL => {
                    let ptr_to_inner =
                        atom_type as *const libpd_sys::t_atom as *mut libpd_sys::t_atom;
                    let sym: *const std::os::raw::c_char =
                        unsafe { libpd_sys::libpd_get_symbol(ptr_to_inner) };
                    let result = unsafe { CStr::from_ptr(sym) };
                    Atom::Symbol(result.to_str().unwrap().to_owned())
                }
                // TODO: Also missing double.. lets see how to cover these.
                _ => unimplemented!(),
            })
            .collect::<Vec<Atom>>()
    };
}

// TODO: Find out how to correctly export these.
pub(crate) use {make_atom_list_from_t_atom_list, make_t_atom_list_from_atom_list};
