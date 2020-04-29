pub mod de;
pub mod ser;

mod varuint;

pub use de::from_reader;
pub use ser::to_writer;
