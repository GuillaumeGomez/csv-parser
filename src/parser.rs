use nom::*;
use std::str;
use std::io::prelude::*;
use std::fs::File;

// http://rust.unhandledexpression.com/nom/macro.escaped!.html
// http://rust.unhandledexpression.com/nom/
// https://fr.wikipedia.org/wiki/Comma-separated_values
// https://github.com/Geal/nom/wiki/Making-a-new-parser-from-scratch
// https://github.com/Geal/nom
// http://rust.unhandledexpression.com/nom/macro.escaped!.html
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

fn get_column_value<'a>(entry: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    match consume_useless_chars(entry) {
        IResult::Done(out, _) => {
            if out.len() < 1 {
                IResult::Done(b"", b"")
            } else if out[0] as char == '\"' {
                match string_between_quotes(out) {
                    IResult::Done(i, out) => {
                        match consume_useless_chars(i) {
                            IResult::Done(i, _) => {
                                if is_not_cell_end(i[0]) {
                                    panic!("Expected `,`, found `{}`", i[0] as char)
                                } else {
                                    IResult::Done(i, out)
                                }
                            }
                            x => x,
                        }
                    }
                    x => x,
                }
            } else if out[0] as char == '\n' {
                IResult::Done(b"", b"")
            } else {
                get_cell(out)
            }
        },
        x => x,
    }
}

fn get_line_values<'a>(ret: &mut Vec<String>, entry: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    if entry.len() < 1 {
        IResult::Done(b"", b"")
    } else {
        match get_column_value(entry) {
            IResult::Done(in_, out) => {
                if out.len() < 1 && in_.len() < 1 {
                    ret.push(String::new());
                    IResult::Done(b"", b"")
                } else {
                    if let Ok(s) = str::from_utf8(out) {
                        ret.push(s.to_owned());
                    }
                    if in_.len() > 0 && in_[0] as char != '\n' {
                        match take!(in_, 1) {
                            IResult::Done(in_, _) => get_line_values(ret, in_),
                            x => x,
                        }
                    } else {
                        IResult::Done(b"", b"")
                    }
                }
            }
            ref n if n.is_incomplete() => {
                if let Some(out) = n.remaining_input() {
                    if out.len() < 1 {
                        IResult::Done(b"", b"")
                    } else {
                        if let Ok(s) = str::from_utf8(out) {
                            ret.push(s.to_owned());
                        }
                        IResult::Done(b"", b"")
                    }
                } else {
                    IResult::Done(b"", b"")
                }
            }
            x => x,
        }
    }
}

fn get_lines_values(ret: &mut Vec<Vec<String>>, entry: &[u8]) {
    match get_line(entry) {
        IResult::Done(in_, out) => {
            let mut line = vec!();

            get_line_values(&mut line, out);
            if line.len() < 1 {
                return;
            }
            ret.push(line);
            if in_.len() > 0 && in_[0] as char == '\n' {
                match take!(in_, 1) {
                    IResult::Done(in_, _) => get_lines_values(ret, in_),
                    _ => {},
                }
                get_lines_values(ret, in_)
            }
        }
        _ => {}
    }
}

pub fn parse_csv_from_slice(entry: &[u8]) -> Vec<Vec<String>> {
    let mut ret = vec!();

    get_lines_values(&mut ret, entry);
    ret
}

pub fn parse_csv_from_file(filename: &str) -> Vec<Vec<String>> {
    let mut f = File::open(filename).unwrap();
    let mut buffer = vec!();

    f.read_to_end(&mut buffer).unwrap();
    parse_csv_from_slice(&buffer)
}

pub fn parse_csv(entry: &str) -> Vec<Vec<String>> {
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
    let mut cells = vec!();

    get_line_values(&mut cells, b"\"nom\",,age\n");
    assert_eq!(cells, vec!("nom".to_owned(), "".to_owned(), "age".to_owned()));
    let mut cells = vec!();
    get_line_values(&mut cells, b"\"nom\",age,\n");
    assert_eq!(cells, vec!("nom".to_owned(), "age".to_owned(), "".to_owned()));
    let mut cells = vec!();
    get_line_values(&mut cells, b"\"nom\",age,,\"hoho\",,end\n");
    assert_eq!(cells, vec!("nom".to_owned(), "age".to_owned(), "".to_owned(), "hoho".to_owned(), "".to_owned(), "end".to_owned()));
}

#[test]
fn check_get_lines_values() {
    let f = b"\"nom\",age\ncarles,30\nlaure,28\n";
    let mut cells = vec!();

    get_lines_values(&mut cells, f);
    assert_eq!(cells, vec!(
                           vec!("nom".to_owned(), "age".to_owned()),
                           vec!("carles".to_owned(), "30".to_owned()),
                           vec!("laure".to_owned(), "28".to_owned())));
}
