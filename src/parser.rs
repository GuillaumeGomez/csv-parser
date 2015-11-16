use nom::{IResult, not_line_ending, line_ending};

fn csv_line(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
	terminated!(input, separated_list!(is_not_bytes!(&b"\n\r,"[..]), not_line_ending), line_ending)
}

#[test]
fn check_file() {
	let f = b"nom,age\ncarles,30\nlaure,28";

	csv_line(f);
}
