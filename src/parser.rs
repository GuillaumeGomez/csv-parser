use nom::*;
use std::str;

// http://rust.unhandledexpression.com/nom/macro.escaped!.html
// http://rust.unhandledexpression.com/nom/
// https://fr.wikipedia.org/wiki/Comma-separated_values
// https://github.com/Geal/nom/wiki/Making-a-new-parser-from-scratch
// https://github.com/Geal/nom
// http://rust.unhandledexpression.com/nom/macro.escaped!.html
named!(string_between_quotes, delimited!(char!('\"'), is_not!("\""), char!('\"')));
named!(get_cell, take_until!(","));
named!(get_line, take_until!("\n"));
named!(consume_useless_chars, take_while!(is_whitespace));

fn is_whitespace(c: u8) -> bool {
    c as char == ' ' || c as char == '\t'
}

fn get_column_value<'a>(entry: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    match consume_useless_chars(entry) {
        IResult::Done(out, _) => {
            if out.len() < 1 {
                IResult::Done(b"", b"")
            } else if out[0] as char == '\"' {
                match string_between_quotes(out) {
                    IResult::Done(i, _) => {
                        match consume_useless_chars(i) {
                            IResult::Done(i, _) => {
                                match get_cell(i) {
                                    IResult::Done(i, _) => IResult::Done(i, out),
                                    // should return the input and keep the out
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
    match get_column_value(entry) {
        IResult::Done(in_, out) => {
            if out.len() < 1 {
                IResult::Done(b"", b"")
            } else {
                if let Ok(s) = str::from_utf8(out) {
                    ret.push(s.to_owned());
                }
                get_line_values(ret, in_)
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

fn get_lines_values(ret: &mut Vec<Vec<String>>, entry: &[u8]) {
    match get_line(entry) {
        IResult::Done(in_, out) => {
            let mut line = vec!();

            get_line_values(&mut line, out);
            if line.len() < 1 {
                return;
            }
            ret.push(line);
            get_lines_values(ret, in_)
        }
        _ => {}
    }
}

#[test]
fn check_file() {
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
fn check_line() {
    let f = b"\"nom\",age\ncarles,30\nlaure,28\n";

    match get_line(f) {
        IResult::Done(_, out) => assert_eq!(out, b"\"nom\",age"),
        IResult::Incomplete(x) => panic!("incomplete: {:?}", x),
        IResult::Error(e) => panic!("error: {:?}", e),
    }
}

#[test]
fn check_get_line() {
    let f = b"\"nom\",age\ncarles,30\nlaure,28\n";
    let mut cells = vec!();

    get_line_values(&mut cells, f);
    assert_eq!(cells, vec!("nom".to_owned(), "age".to_owned()));
}
