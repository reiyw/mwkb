use std::{thread, time};

use failure::Error;
use indicatif::ProgressBar;
use reqwest::Client;
use url::Url;

use data::Data;

pub fn ensure_endpoint_api_url(url: &str) -> Result<String, Error> {
    let parsed = Url::parse(url)?;
    Ok(format!(
        "{}://{}/api.php",
        parsed.scheme(),
        parsed.host_str().unwrap()
    ))
}

pub fn ensure_endpoint_index_url(url: &str) -> Result<String, Error> {
    let parsed = Url::parse(url)?;
    Ok(format!(
        "{}://{}/index.php",
        parsed.scheme(),
        parsed.host_str().unwrap()
    ))
}

pub fn retrieve_all_markuped_text(url: &str, data_dir: &str) -> Result<(), Error> {
    let data = Data::new(data_dir);
    // already retrieved
    let ids = data.make_pageid_set_from_markuped_text_files()?;
    // to retrieve
    let titles = data.load_titles()?;
    // cache
    let client = reqwest::Client::new();
    let pb = ProgressBar::new(titles.len() as u64);
    for title in titles {
        if !ids.contains(&title.id) {
            let text = request_markuped_text(url, &title.name[..], &client)?;
            data.save_markuped_text(title.id, &text[..])?;
            thread::sleep(time::Duration::from_secs(1));
        }
        pb.inc(1)
    }
    pb.finish_with_message("done");
    Ok(())
}

fn request_markuped_text(url: &str, title: &str, client: &Client) -> Result<String, Error> {
    let query = &[("action", "raw"), ("title", title), ("utf8", "true")];
    for _ in 0..5 {
        let mut res = client.get(url).query(query).send();
        match res {
            Ok(ref mut resp) => {
                return Ok(resp.text()?);
            }
            Err(e) => {
                eprintln!("unexpected error: {:?}", e);
                eprintln!("retry after 1 sec...");
                thread::sleep(time::Duration::from_secs(1));
            }
        }
    }
    Err(format_err!("retried 5 times but can't receive"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_endpoint_api_url() -> Result<(), Error> {
        let url_expected = "https://minecraft.gamepedia.com/api.php".to_string();
        let url_pre = "https://minecraft.gamepedia.com";
        assert_eq!(ensure_endpoint_api_url(url_pre)?, url_expected);
        let url_pre = "https://minecraft.gamepedia.com/index.php";
        assert_eq!(ensure_endpoint_api_url(url_pre)?, url_expected);
        let url_pre = "https://minecraft.gamepedia.com/Minecraft_Wiki";
        assert_eq!(ensure_endpoint_api_url(url_pre)?, url_expected);
        Ok(())
    }

    #[test]
    fn test_ensure_endpoint_index_url() -> Result<(), Error> {
        let url_expected = "https://minecraft.gamepedia.com/index.php".to_string();
        let url_pre = "https://minecraft.gamepedia.com";
        assert_eq!(ensure_endpoint_index_url(url_pre)?, url_expected);
        let url_pre = "https://minecraft.gamepedia.com/api.php";
        assert_eq!(ensure_endpoint_index_url(url_pre)?, url_expected);
        let url_pre = "https://minecraft.gamepedia.com/Minecraft_Wiki";
        assert_eq!(ensure_endpoint_index_url(url_pre)?, url_expected);
        Ok(())
    }

    #[test]
    fn test() {
        retrieve_all_markuped_text("2018-11-16_minecraft");
        assert!(false);
    }
}
