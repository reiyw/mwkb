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

use std::{cmp, fs, thread, time};
use std::path::Path;

use failure::Error;
use regex::Regex;

pub mod util;
mod parser;

#[derive(Serialize, Deserialize, Debug)]
pub struct Title {
    id: u32,
    // see: https://www.mediawiki.org/wiki/Manual:Namespace/ja#%E7%B5%84%E3%81%BF%E8%BE%BC%E3%81%BF%E3%81%AE%E5%90%8D%E5%89%8D%E7%A9%BA%E9%96%93
    ns: i32,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Page {
    title: Title,
    text: String,
}

#[derive(Deserialize, Debug)]
struct MWError {
    code: String,
    info: String,
}

#[derive(Deserialize, Debug)]
struct MWContinue {
    apcontinue: String,
}

#[derive(Deserialize, Debug)]
struct MWPage {
    pageid: u32,
    // see: https://www.mediawiki.org/wiki/Manual:Namespace/ja#%E7%B5%84%E3%81%BF%E8%BE%BC%E3%81%BF%E3%81%AE%E5%90%8D%E5%89%8D%E7%A9%BA%E9%96%93
    ns: i32,
    title: String,
}

#[derive(Deserialize, Debug)]
struct MWQuery {
    allpages: Vec<MWPage>,
}

#[derive(Deserialize, Debug)]
struct MWAllpagesApiResponse {
    error: Option<MWError>,
    #[serde(rename = "continue")]
    _continue: Option<MWContinue>,
    query: Option<MWQuery>,
}

/// Parse time from maxlag info like this: "Waiting for a database server: 0 seconds lagged."
fn parse_maxlag_waiting_time(info: &str) -> u64 {
    lazy_static! {
        static ref MAXLAG_RE: Regex = Regex::new(r"(?P<time>\d+) seconds lagged.$").unwrap();
    }
    let caps = MAXLAG_RE.captures(info).unwrap();
    caps["time"].parse().unwrap()
}

/// Return Vec of Title and the following name of the last Title
///
/// You can proceed next request by passing the last name to `from`.
fn request_titles_partially(
    url: &str,
    limit: u32,
    from: Option<&str>,
    maxlag: i32,
) -> Result<(Vec<Title>, Option<String>), Error> {
    let limit = if limit > 500 { 500 } else { limit };
    let query = &[
        ("action", "query"),
        ("list", "allpages"),
        ("format", "json"),
        ("utf8", "true"),
        ("aplimit", &limit.to_string()),
        ("apfrom", from.unwrap_or("")),
        ("maxlag", &maxlag.to_string()),
    ];
    let client = reqwest::Client::new();
    for _ in 0..5 {
        let json: MWAllpagesApiResponse = client.get(url).query(query).send()?.json()?;
        match json.error {
            Some(e) => match &e.code[..] {
                "maxlag" => {
                    let secs = cmp::max(1, parse_maxlag_waiting_time(&e.info));
                    let dur = time::Duration::from_secs(secs);
                    eprintln!("maxlag error: retry after {} secs", secs);
                    thread::sleep(dur);
                    continue;
                }
                _ => return Err(format_err!("unexpected API error: {:?}", e)),
            },
            None => {
                let titles: Vec<Title> = json
                    .query
                    .unwrap()
                    .allpages
                    .into_iter()
                    .map(|p| Title {
                        id: p.pageid,
                        ns: p.ns,
                        name: p.title,
                    }).collect();
                eprintln!(
                    "retrieved {} titles: \"{}\" ... \"{}\"",
                    titles.len(),
                    titles.first().unwrap().name,
                    titles.last().unwrap().name
                );
                let apcontinue = json._continue.map(|p| p.apcontinue);
                return Ok((titles, apcontinue));
            }
        }
    }
    Err(format_err!("retried 5 times but can't receive"))
}

pub fn load_titles<P: AsRef<Path>>(filepath: P) -> Result<Vec<Title>, Error> {
    let mut rdr = csv::Reader::from_path(filepath)?;
    let mut titles = Vec::new();
    for res in rdr.deserialize() {
        let title: Title = res?;
        titles.push(title);
    }
    Ok(titles)
}

pub fn save_titles<P: AsRef<Path>>(titles: &Vec<Title>, filepath: P) -> Result<(), Error> {
    let mut wtr = csv::Writer::from_path(filepath)?;
    for title in titles {
        wtr.serialize(title)?;
    }
    Ok(())
}

fn request_next_title(title: &str, url: &str) -> Result<Option<String>, Error> {
    let (_, next_title) = request_titles_partially(url, 1, Some(title), 5)?;
    Ok(next_title)
}

/// Populate parameter `titles` and return Result to save progress
///
/// You can resume retrieving by passing existing titles.
pub fn retrieve_all_titles(titles: &mut Vec<Title>, url: &str) -> Result<(), Error> {
    let limit = 500;
    // recommended. see: https://www.mediawiki.org/wiki/Manual:Maxlag_parameter/ja
    let maxlag = 5;
    // for allocation
    let mut next_title = match titles.last() {
        Some(title) => request_next_title(&title.name[..], url)?,
        None => None,
    };
    // wait 1 sec per each request
    let interval = time::Duration::from_secs(1);
    loop {
        let res =
            request_titles_partially(url, limit, next_title.as_ref().map(String::as_str), maxlag)?;
        let partial_titles = res.0;
        titles.extend(partial_titles);
        next_title = res.1;
        if next_title.is_some() {
            thread::sleep(interval);
        } else {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_titles_partially() {
        let url = "https://minecraft.gamepedia.com/api.php";
        let limit = 3;
        let from = "Ore";
        let maxlag = 5;
        let (titles, next_title) = request_titles_partially(url, limit, Some(from), maxlag)
            .expect("Error in calling Allpages API");
        assert_eq!(titles.len(), 3);
        assert_eq!(titles[0].id, 3020);
        assert_eq!(titles[0].ns, 0);
        assert_eq!(titles[0].name, "Ore");
        assert_eq!(titles[1].id, 24709);
        assert_eq!(titles[1].ns, 0);
        assert_eq!(titles[1].name, "Ore/video");
        assert_eq!(titles[2].id, 7315);
        assert_eq!(titles[2].ns, 0);
        assert_eq!(titles[2].name, "Ore Block");
        assert_eq!(next_title, Some("Ores".to_string()));

        // first element
        let limit = 1;
        let from = None;
        let (titles, next_title) = request_titles_partially(url, limit, from, maxlag)
            .expect("Error in calling Allpages API");
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0].id, 89618);
        assert_eq!(titles[0].ns, 0);
        assert_eq!(titles[0].name, "'Tis but a scratch");
        assert_eq!(next_title, Some("...has_become_the_master".to_string()));

        // last element
        let from = "Žodynas/lt";
        let (titles, next_title) = request_titles_partially(url, limit, Some(from), maxlag)
            .expect("Error in calling Allpages API");
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0].id, 21373);
        assert_eq!(titles[0].ns, 0);
        assert_eq!(titles[0].name, "Žodynas/lt");
        assert!(next_title.is_none());
    }

    #[test]
    fn test_request_titles_partially_fail_maxlag_error() {
        let url = "https://minecraft.gamepedia.com/api.php";
        let limit = 3;
        let from = "Ore";
        let maxlag = -1;
        let res = request_titles_partially(url, limit, Some(from), maxlag);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_maxlag_waiting_time() {
        assert_eq!(
            parse_maxlag_waiting_time("Waiting for a database server: 0 seconds lagged."),
            0
        );
        assert_eq!(
            parse_maxlag_waiting_time("Waiting for a database server: 10 seconds lagged."),
            10
        );
    }

    #[test]
    fn test_title_file_io() {
        let tempfile = "tmp.csv";
        let titles_expected = vec![
            Title {
                id: 0,
                ns: 0,
                name: "a".to_string(),
            },
            Title {
                id: 1,
                ns: 0,
                name: "b".to_string(),
            },
        ];
        save_titles(&titles_expected, tempfile);
        let res = load_titles(tempfile);
        assert!(fs::remove_file(tempfile).is_ok());
        let titles_actual = res.unwrap();
        assert_eq!(titles_actual.len(), titles_expected.len());
        assert_eq!(titles_actual[0].id, titles_expected[0].id);
        assert_eq!(titles_actual[0].ns, titles_expected[0].ns);
        assert_eq!(titles_actual[0].name, titles_expected[0].name);
        assert_eq!(titles_actual[1].id, titles_expected[1].id);
        assert_eq!(titles_actual[1].ns, titles_expected[1].ns);
        assert_eq!(titles_actual[1].name, titles_expected[1].name);
    }

    #[test]
    fn test_request_next_title() -> Result<(), Error> {
        let url = "https://minecraft.gamepedia.com/api.php";
        assert_eq!(
            request_next_title("'Tis but a scratch", url)?,
            Some("...has_become_the_master".to_string())
        );
        // last element
        assert_eq!(request_next_title("Žodynas/lt", url)?, None);
        Ok(())
    }
}
