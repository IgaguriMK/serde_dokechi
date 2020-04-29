//! `serde_dokechi` is a serializer / deserializer library focus on only serialized binary size.
//!
//! Minimum supprted Rust version is `1.40.0 (2019-12-19)`.

pub mod de;
pub mod ser;

mod varuint;

pub use de::from_reader;
pub use ser::to_writer;
