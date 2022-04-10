// TODO: Mention why only queued versions are implemented in the main doc.

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    // Group of too restrictive lints
    clippy::undocumented_unsafe_blocks,
    clippy::as_conversions,
    clippy::integer_arithmetic,
    clippy::float_arithmetic,
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::enum_glob_use,
    clippy::wildcard_enum_match_arm,
    clippy::pattern_type_mismatch,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::multiple_crate_versions,
    clippy::default_numeric_fallback,
    clippy::map_err_ignore,


    // We decided that we're ok with expect
    clippy::expect_used,

    // Too restrictive for the current style
    clippy::missing_inline_in_public_items,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::module_name_repetitions,
    clippy::unseparated_literal_suffix,
    clippy::self_named_module_files,

    // Allowed lints related to cargo
    // (comment these out if you'd like to improve Cargo.toml)
    clippy::wildcard_dependencies,
    clippy::redundant_feature_names,
    clippy::cargo_common_metadata,

    // Comment these out when writing docs
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,

    // Comment these out before submitting a PR
    clippy::todo,
    clippy::panic_in_result_fn,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,
)]

pub mod convenience;
pub mod error;
pub(crate) mod helpers;
pub mod mirror;
pub mod types;
