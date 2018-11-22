extern crate failure;
extern crate mwkb;

use std::env;

use failure::Error;

use mwkb::api::ensure_endpoint_api_url;
use mwkb::data::Data;
use mwkb::title::retrieve_all_titles;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let url = ensure_endpoint_api_url(&args[1])?;
    let data = Data::new(&args[2]);
    let mut titles = if data.title_file.exists() {
        data.load_titles()?
    } else {
        Vec::new()
    };
    let res = retrieve_all_titles(&mut titles, &url[..]);
    data.save_titles(&titles)?;

    res
}
