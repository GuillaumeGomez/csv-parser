#[macro_use]
extern crate nom;

pub use self::parser::{
    parse_csv_from_slice,
    parse_csv_from_file,
    parse_csv,
};

mod parser;
