extern crate csv;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate parse_wiki_text;
extern crate regex;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate url;

pub mod api;
pub mod parser;
pub mod title;
