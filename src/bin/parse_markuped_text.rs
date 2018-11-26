extern crate failure;
extern crate mwkb;

use std::env;

use failure::Error;

use mwkb::parser::parse_all_markuped_text;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    parse_all_markuped_text(&args[1])
}
