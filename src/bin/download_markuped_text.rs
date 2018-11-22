extern crate failure;
extern crate mwkb;

use std::env;

use failure::Error;

use mwkb::api::{ensure_endpoint_index_url, retrieve_all_markuped_text};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let url = ensure_endpoint_index_url(&args[1])?;
    retrieve_all_markuped_text(&url[..], &args[2])
}
