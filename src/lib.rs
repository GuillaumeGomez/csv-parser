#[macro_use]
extern crate nom;

pub use self::error::{
    CharError,
    CsvError,
    Position,
    SizeError,
};
pub use self::parser::{
    parse_csv_from_slice,
    parse_csv_from_file,
    parse_csv,
};

mod error;
mod parser;
