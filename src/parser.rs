use std::str;
use std::io::prelude::*;
use std::fs::File;

use nom::*;

use error::*;

named!(string_between_quotes, delimited!(char!('\"'), is_not!("\""), char!('\"')));
named!(get_cell, take_while!(is_not_cell_end));
named!(get_line, take_until!("\n"));
named!(consume_useless_chars, take_while!(is_whitespace));

fn is_whitespace(c: u8) -> bool {
    c as char == ' ' || c as char == '\t'
}

fn is_not_cell_end(c: u8) -> bool {
    c as char != ',' && c as char != '\n'
}



fn get_column_value(input: &[u8], pos: Position) -> IResult<&[u8], &[u8], CsvError> {
    let (i, cell) = try_parse!(input,
        fix_error!(CsvError,
            preceded!(
            opt!(consume_useless_chars),
                alt!(
                    string_between_quotes
                  | get_cell
                )
            )
        )
    );

    if i.len() == 0 {
        IResult::Incomplete(Needed::Unknown)
    } else if is_not_cell_end(i[0]) {
      let p = Position { line: pos.line, column: pos.column + input.offset(i) };
        IResult::Error(Err::Code(ErrorKind::Custom(
            CsvError::InvalidCharacter(CharError::new(',', i[0] as char, &p))
        )))
    } else {
      IResult::Done(i, cell)
    }
}

fn get_string_column_value(input: &[u8], pos: Position) -> IResult<&[u8], String, CsvError> {
    map_res!(input,
        map_res!(
            dbg_dmp!(
                apply!(get_column_value, Position::new(pos.line, pos.column))
            ),
            from_utf8
        ),
        |d| {
            str::FromStr::from_str(d)
        }
    )
}


fn comma_then_column(input: &[u8], pos: Position) -> IResult<&[u8], String, CsvError> {
    preceded!(input,
        fix_error!(CsvError, char!(',')),
        apply!(get_string_column_value, Position::new(pos.line, pos.column))
    )
}

use std::str::from_utf8;
fn get_line_values<'a>(entry: &'a[u8],ret: &mut Vec<String>, line: usize) -> IResult<&'a[u8], &'a[u8], CsvError> {
    if entry.len() == 0 {
        IResult::Done(b"", b"")
    } else {
        let (i, col) = try_parse!(entry, apply!(get_string_column_value, Position::new(line, ret.len())));
        ret.push(col);

        match fix_error!(i, CsvError, terminated!(
            many0!(
                apply!(comma_then_column, Position::new(line, ret.len()))
            ),
            char!('\n')
        )) {
            IResult::Done(i, v) => {
              ret.extend(v);
              IResult::Done(i, &entry[..entry.offset(i)])
            },
            IResult::Incomplete(i) => IResult::Incomplete(i),
            IResult::Error(e)      => IResult::Error(e)
        }
    }
}


fn get_lines_values(mut ret: Vec<Vec<String>>, entry: &[u8]) -> Result<Vec<Vec<String>>, CsvError> {
    let mut input = entry;
    let mut line  = 0;
    loop {
        let mut v: Vec<String> = Vec::new();
        match get_line_values(input, &mut v, line) {
            IResult::Error(Err::Code(ErrorKind::Custom(e))) => return Err(e),
            IResult::Error(_)                               => return Err(CsvError::GenericError),
            IResult::Incomplete(_)                          => {
              // we reached the end of file?
              break
            }
            IResult::Done(i,_)                              => {
                input = i;
                line += 1;
                ret.push(v);
                if input.len() == 0 {
                    break;
                }
            },
        }
    }

    Ok(ret)
}

pub fn parse_csv_from_slice(entry: &[u8]) -> Result<Vec<Vec<String>>, CsvError> {
    get_lines_values(vec!(), entry)
}

pub fn parse_csv_from_file(filename: &str) -> Result<Vec<Vec<String>>, CsvError> {
    let mut f = File::open(filename).unwrap();
    let mut buffer = vec!();

    f.read_to_end(&mut buffer).unwrap();
    parse_csv_from_slice(&buffer)
}

pub fn parse_csv(entry: &str) -> Result<Vec<Vec<String>>, CsvError> {
    parse_csv_from_slice(entry.as_bytes())
}

#[test]
fn check_string_between_quotes() {
    let f = b"\"nom\",age\ncarles,30\nlaure,28\n";

    match string_between_quotes(f) {
        IResult::Done(in_, out) => {
            assert_eq!(out, b"nom");
            assert_eq!(in_, b",age\ncarles,30\nlaure,28\n");
        },
        IResult::Incomplete(x) => panic!("incomplete: {:?}", x),
        IResult::Error(e) => panic!("error: {:?}", e),
    }
}

#[test]
fn check_get_cell() {
    let f = b"age\ncarles,30\n";
    let g = b"age2,carles,30\n";

    match get_cell(f) {
        IResult::Done(_, out) => assert_eq!(out, b"age"),
        IResult::Incomplete(x) => panic!("incomplete: {:?}", x),
        IResult::Error(e) => panic!("error: {:?}", e),
    }
    match get_cell(g) {
        IResult::Done(_, out) => assert_eq!(out, b"age2"),
        IResult::Incomplete(x) => panic!("incomplete: {:?}", x),
        IResult::Error(e) => panic!("error: {:?}", e),
    }
}


#[test]
fn check_get_line() {
    let f = b"\"nom\",age\ncarles,30\nlaure,28\n";

    match get_line(f) {
        IResult::Done(_, out) => assert_eq!(out, b"\"nom\",age"),
        IResult::Incomplete(x) => panic!("incomplete: {:?}", x),
        IResult::Error(e) => panic!("error: {:?}", e),
    }
}

#[test]
fn check_get_line_values() {
    // no terminator, this is not a line
    //let mut cells = vec!();
    //get_line_values(&mut cells, b"\"nom\",,age", 0);
    //assert_eq!(cells, vec!("nom".to_owned(), "".to_owned(), "age".to_owned()));

    let mut cells = vec!();
    let res = get_line_values(b"\"nom\",,age\n", &mut cells, 0);
    println!("res: {:?}", res);
    assert_eq!(cells, vec!("nom".to_owned(), "".to_owned(), "age".to_owned()));

    let mut cells = vec!();
    get_line_values(b"\"nom\",age,\n", &mut cells, 0);
    assert_eq!(cells, vec!("nom".to_owned(), "age".to_owned(), "".to_owned()));

    let mut cells = vec!();
    get_line_values(b"\"nom\",age,,\"hoho\",,end\n", &mut cells, 0);
    assert_eq!(cells, vec!("nom".to_owned(), "age".to_owned(), "".to_owned(), "hoho".to_owned(), "".to_owned(), "end".to_owned()));

    let mut cells = vec!();
    let e = get_line_values(b"\"nom\" ,age,\"hoho\"", &mut cells, 0);
    assert_eq!(e,
        IResult::Error(Err::Code(ErrorKind::Custom(
            CsvError::InvalidCharacter(CharError::new(',', ' ', &Position::new(0, 5)))
        )))
    );

}

#[test]
fn check_get_lines_values() {
    let f = b"\"nom\",age\ncarles,30\nlaure,28\n";

    assert_eq!(get_lines_values(vec!(), f),
               Ok(vec!(
                       vec!("nom".to_owned(), "age".to_owned()),
                       vec!("carles".to_owned(), "30".to_owned()),
                       vec!("laure".to_owned(), "28".to_owned()))));
}

#[test]
fn check_parse_csv() {
    let f = "\"nom\",age\ncarles,30\nlaure,28\n";

    assert_eq!(parse_csv(f),
               Ok(vec!(
                       vec!("nom".to_owned(), "age".to_owned()),
                       vec!("carles".to_owned(), "30".to_owned()),
                       vec!("laure".to_owned(), "28".to_owned()))));
}
