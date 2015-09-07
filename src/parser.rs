use nom::{IResult, digit, multispace};

struct CSV {
	keys: Vec<&str>,
	values: Vec<Vec<&str>>,
}

terminated!(separated_list!(filter!(check_characters(b"\n\r,")), not_line_ending), line_ending, a);

fn check_characters(characters: &[u8], data: &[u8]) -> bool {
	for i in 0..data.len() {
		for c in characters {
			if data[i] == c {
				return false;
			}
		}
	}
	true
}

/*fn fill_keys(input: &[u8]) -> IResult<&[u8], Vec<&str>> {

}*/

#[test]
fn check_file() {
	let f = b"nom,age\ncarles,30\nlaure,28";

	separated_list!(f);
}