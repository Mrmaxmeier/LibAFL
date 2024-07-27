//! Sugar API to simplify the life of users of `LibAFL` that just want to fuzz.
/*! */
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]

#[allow(clippy::ignored_unit_patterns)]
pub mod inmemory;
pub use inmemory::InMemoryBytesCoverageSugar;

#[cfg(target_os = "linux")]
#[allow(clippy::ignored_unit_patterns)]
pub mod qemu;
#[cfg(target_os = "linux")]
pub use qemu::QemuBytesCoverageSugar;

#[cfg(target_family = "unix")]
#[allow(clippy::ignored_unit_patterns)]
pub mod forkserver;
#[cfg(target_family = "unix")]
pub use forkserver::ForkserverBytesCoverageSugar;

/// Default timeout for a run
pub const DEFAULT_TIMEOUT_SECS: u64 = 1200;
/// Default cache size for the corpus in memory.
/// Anything else will be on disk.
pub const CORPUS_CACHE_SIZE: usize = 4096;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// The sugar python module
#[cfg(feature = "python")]
#[pymodule]
#[pyo3(name = "libafl_sugar")]
pub fn python_module(py: Python, m: &PyModule) -> PyResult<()> {
    inmemory::pybind::register(py, m)?;
    #[cfg(target_os = "linux")]
    {
        qemu::pybind::register(py, m)?;
    }
    #[cfg(unix)]
    {
        forkserver::pybind::register(py, m)?;
    }
    Ok(())
}
