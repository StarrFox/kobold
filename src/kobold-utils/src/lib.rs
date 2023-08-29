//! Shared utility code throughout the Kobold project.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

pub use anyhow;
pub use binrw;
pub use libdeflater;

pub mod align;
pub mod hash;
