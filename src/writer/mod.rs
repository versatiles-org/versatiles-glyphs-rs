//! Provides implementations of the [`Writer`](writer::Writer) trait for various output targets.

#[cfg(test)]
pub mod dummy;
mod file;
mod tar;
mod traits;

pub use file::FileWriter;
pub use tar::TarWriter;
pub use traits::Writer;
