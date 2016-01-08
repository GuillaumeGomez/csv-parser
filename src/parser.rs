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

    /*match consume_useless_chars(entry) {
        IResult::Done(out, _) => {
            if out.len() < 1 {
                Ok(IResult::Done(b"", b""))
            } else if out[0] as char == '\"' {
                match string_between_quotes(out) {
                    IResult::Done(i, out) => {
                        if is_not_cell_end(i[0]) {
                            Err(CsvError::InvalidCharacter(CharError::new(',', i[0] as char, &pos)))
                            //panic!("Expected `,`, found `{}`", i[0] as char)
                        } else {
                            Ok(IResult::Done(i, out))
                        }
                    }
                    x => Ok(x),
                }
            } else if out[0] as char == '\n' {
                Ok(IResult::Done(b"", b""))
            } else {
                Ok(get_cell(out))
            }
        },
        x => Ok(x),
    }*/

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
fn get_line_values<'a>(ret: &mut Vec<String>, entry: &'a [u8],
    line: usize) -> IResult<&'a [u8], &'a [u8], CsvError> {
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

        /*
        match get_column_value(entry, Position::new(line, ret.len())) {
          IResult::Done(in_, out) => {
                if out.len() < 1 && in_.len() < 1 {
                    ret.push(String::new());
                    IResult::Done(b"", b"")
                } else {
                    if let Ok(s) = str::from_utf8(out) {
                        ret.push(s.to_owned());
                    }
                    if in_.len() > 0 && in_[0] as char != '\n' {
                        match fix_error!(in_, CsvError, take!(1)) {
                            IResult::Done(in_, _) => get_line_values(ret, in_, line),
                            x => x,
                        }
                    } else {
                        IResult::Done(b"", b"")
                    }
                }
            }
            x => x
              */
            /*
            n if n.is_incomplete() => {
                if let Some(out) = n.remaining_input() {
                    if out.len() < 1 {
                        Ok(IResult::Done(b"", b""))
                    } else {
                        if let Ok(s) = str::from_utf8(out) {
                            ret.push(s.to_owned());
                        }
                        Ok(IResult::Done(b"", b""))
                    }
                } else {
                    Ok(IResult::Done(b"", b""))
                }
            }
            x => Ok(x),
            */
       // }
}

fn get_lines_values(mut ret: Vec<Vec<String>>, entry: &[u8]) -> Result<Vec<Vec<String>>, CsvError> {
    match get_line(entry) {
        IResult::Done(in_, out) => {
            let mut line = vec!();

            match get_line_values(&mut line, out, ret.len()) {
                Err(e) => return Err(e),
                _ => {}
            }
            // we stop at first empty line
            if line.len() < 1 {
                return Ok(ret)
            }
            if ret.len() > 0 && line.len() != ret[0].len() {
                panic!("Line `{}` has `{}` value{}, `{}` {} expected.",
                       ret.len(), line.len(), if line.len() > 1 { "s" } else { "" },
                       ret[0].len(), if ret[0].len() > 1 { "were" } else { "was" });
            }
            ret.push(line);
            if in_.len() > 0 && in_[0] as char == '\n' {
                match take!(in_, 1) {
                    IResult::Done(in_, _) => get_lines_values(ret, in_),
                    _ => Ok(ret),
                }
            } else {
                Ok(ret)
            }
        }
        _ => Ok(ret),
    }
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
    let res = get_line_values(&mut cells, b"\"nom\",,age\n", 0);
    println!("res: {:?}", res);
    assert_eq!(cells, vec!("nom".to_owned(), "".to_owned(), "age".to_owned()));

    let mut cells = vec!();
    get_line_values(&mut cells, b"\"nom\",age,\n", 0);
    assert_eq!(cells, vec!("nom".to_owned(), "age".to_owned(), "".to_owned()));

    let mut cells = vec!();
    get_line_values(&mut cells, b"\"nom\",age,,\"hoho\",,end\n", 0);
    assert_eq!(cells, vec!("nom".to_owned(), "age".to_owned(), "".to_owned(), "hoho".to_owned(), "".to_owned(), "end".to_owned()));

    let mut cells = vec!();
    let e = get_line_values(&mut cells, b"\"nom\" ,age,\"hoho\"", 0);
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
