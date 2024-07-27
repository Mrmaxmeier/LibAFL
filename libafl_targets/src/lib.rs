//! `libafl_targets` contains runtime code, injected in the target itself during compilation.
#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;
#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

include!(concat!(env!("OUT_DIR"), "/constants.rs"));

#[cfg(any(
    feature = "sancov_pcguard_edges",
    feature = "sancov_pcguard_hitcounts",
    feature = "sancov_ngram4",
    feature = "sancov_ctx"
))]
pub mod sancov_pcguard;
#[cfg(any(
    feature = "sancov_pcguard_edges",
    feature = "sancov_pcguard_hitcounts",
    feature = "sancov_ngram4",
    feature = "sancov_ctx"
))]
pub use sancov_pcguard::*;

#[cfg(any(feature = "sancov_cmplog", feature = "sancov_value_profile"))]
pub mod sancov_cmp;
#[cfg(any(feature = "sancov_cmplog", feature = "sancov_value_profile"))]
pub use sancov_cmp::*;

/// Module containing bindings to the various sanitizer interface headers
#[cfg(feature = "sanitizer_interfaces")]
#[allow(clippy::mixed_attributes_style)]
pub mod sanitizer_ifaces {
    #![allow(non_snake_case)]
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]
    #![allow(unused)]
    #![allow(improper_ctypes)]
    #![allow(clippy::unreadable_literal)]
    #![allow(missing_docs)]
    #![allow(missing_debug_implementations)]
    #![allow(unused_qualifications)]
    include!(concat!(env!("OUT_DIR"), "/sanitizer_interfaces.rs"));
}

#[cfg(feature = "libfuzzer")]
pub mod libfuzzer;
#[cfg(feature = "libfuzzer")]
pub use libfuzzer::*;

#[cfg(feature = "sancov_8bit")]
pub mod sancov_8bit;
#[cfg(feature = "sancov_8bit")]
pub use sancov_8bit::*;

#[cfg(feature = "coverage")]
pub mod coverage;
#[cfg(feature = "coverage")]
pub use coverage::*;

pub mod value_profile;
pub use value_profile::*;

/// The module to hook call instructions
#[cfg(feature = "function-logging")]
pub mod call;
#[cfg(feature = "function-logging")]
pub use call::*;

/// runtime related to comparisons
pub mod cmps;
pub use cmps::*;

#[cfg(feature = "std")]
pub mod drcov;

#[cfg(all(windows, feature = "std", feature = "windows_asan"))]
pub mod windows_asan;
#[cfg(all(windows, feature = "std", feature = "windows_asan"))]
pub use windows_asan::*;

#[cfg(all(unix, feature = "forkserver"))]
pub mod forkserver;
#[cfg(all(unix, feature = "forkserver"))]
pub use forkserver::*;
