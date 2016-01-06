use nom::{IResult, not_line_ending, line_ending};

// http://rust.unhandledexpression.com/nom/macro.escaped!.html
// http://rust.unhandledexpression.com/nom/
// https://fr.wikipedia.org/wiki/Comma-separated_values
// https://github.com/Geal/nom/wiki/Making-a-new-parser-from-scratch
// https://github.com/Geal/nom
fn csv_line(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    terminated!(input, separated_list!(is_not_bytes!(&b"\n\r,"[..]), not_line_ending), line_ending)
}

fn parse_string_between_quotes(input: &str) {
    
}

fn get_data(input: &str) -> Vec<Vec<String>> {

}

#[test]
fn check_file() {
    let f = b"nom,age\ncarles,30\nlaure,28";

    csv_line(f);
}
