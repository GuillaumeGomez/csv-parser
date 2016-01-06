use nom::*;

// http://rust.unhandledexpression.com/nom/macro.escaped!.html
// http://rust.unhandledexpression.com/nom/
// https://fr.wikipedia.org/wiki/Comma-separated_values
// https://github.com/Geal/nom/wiki/Making-a-new-parser-from-scratch
// https://github.com/Geal/nom
// http://rust.unhandledexpression.com/nom/macro.escaped!.html
named!(string_between_quotes, delimited!(char!('\"'), is_not!("\""), char!('\"')));

#[test]
fn check_file() {
    let f = b"\"nom\",age\ncarles,30\nlaure,28";

    match string_between_quotes(f) {
        IResult::Done(_, out) => assert_eq!(out, b"nom"),
        IResult::Incomplete(x) => panic!("incomplete: {:?}", x),
        IResult::Error(e) => panic!("error: {:?}", e),
    }
}
