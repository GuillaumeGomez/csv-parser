use std::cmp::PartialEq;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CsvError {
    InvalidCharacter(CharError),
    InvalidRowLength(SizeError),
    GenericError
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CsvError::InvalidCharacter(ref c) => write!(f, "InvalidCharacter: {}: expected `{}`, got `{}`",
                                                        c.position, c.expected, c.got),
            CsvError::InvalidRowLength(ref c) => write!(f, "InvalidRowLength: {}: expected `{}` element{}, got `{}`",
                                                        c.position, c.nb_elements_expected,
                                                        if c.nb_elements_expected > 1 { "s" } else { "" }, c.nb_elements_got),
            CsvError::GenericError            => write!(f, "GenericError")
        }
    }
}

impl PartialEq for CsvError {
    fn eq(&self, other: &CsvError) -> bool {
        match (self, other) {
            (&CsvError::InvalidCharacter(ref c), &CsvError::InvalidCharacter(ref o)) => c == o,
            (&CsvError::InvalidRowLength(ref c), &CsvError::InvalidRowLength(ref o)) => c == o,
            (&CsvError::GenericError, &CsvError::GenericError)                       => true,
            _ => false,
        }
    }

    fn ne(&self, other: &CsvError) -> bool {
        self.eq(other) == false
    }
}

impl Error for CsvError {
    fn description(&self) -> &str {
        match *self {
            CsvError::InvalidCharacter(_) => "invalid character",
            CsvError::InvalidRowLength(_) => "invalid row length",
            CsvError::GenericError        => "generic parsing error"
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl Position {
    pub fn new(line: usize, column: usize) -> Position {
        Position {
            line: line,
            column: column,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CharError {
    pub expected: char,
    pub got: char,
    pub position: Position,
}

impl CharError {
    pub fn new(expected: char, got: char, pos: &Position) -> CharError {
        CharError {
            expected: expected,
            got: got,
            position: pos.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SizeError {
    pub nb_elements_expected: usize,
    pub nb_elements_got: usize,
    pub position: Position,
}

impl SizeError {
    pub fn new(nb_elements_expected: usize, nb_elements_got: usize,
               pos: &Position) -> SizeError {
        SizeError {
            nb_elements_expected: nb_elements_expected,
            nb_elements_got: nb_elements_got,
            position: pos.clone(),
        }
    }
}
