extern crate failure;
extern crate mwkb;

use std::env;

use failure::Error;

use mwkb::*;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let url = util::ensure_endpoint_url(&args[1])?;
    let data_dir = &args[2];
    let save_path = format!("{}/titles.csv", data_dir);

    eprintln!("retrieve titles from {}", &args[1]);

    let mut titles: Vec<Title> = Vec::new();
    let res = retrieve_all_titles(&mut titles, &url[..]);
    save_titles(&titles, save_path)?;

    res
}
