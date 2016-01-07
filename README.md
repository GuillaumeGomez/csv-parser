# csv-parser

A CSV parser written in Rust with [nom](https://github.com/Geal/nom).

##HOWTO

This library provides 3 functions:

 * parse_csv
 * parse_csv_from_file
 * parse_csv_from_slice

They all return a `Result` containing a vector of vector. The first line of the vector contains each column name.

##Examples

A short one:

```Rust
use csv_parser;

fn main() {
    let csv_to_parse = "\"nom\",age\ncarles,30\nlaure,28\n";
    if let Ok(parsed_csv) = parse_csv(csv_to_parse) {
        // and we're all good!
    }
}
```

You can give a file path as well:

```Rust
use csv_parser;

fn main() {
    let csv_file = "some_file.csv";
    if let Ok(parsed_csv) = parse_csv_from_file(csv_file) {
        // and we're all good!
    }
}
```
