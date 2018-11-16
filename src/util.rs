use reqwest::Url;

use failure::Error;

pub fn ensure_endpoint_url(url: &str) -> Result<String, Error> {
    let _url = Url::parse(url)?;
    match _url.path() {
        "/api.php" => Ok(url.to_string()),
        "/" => Ok(format!("{}/api.php", url)),
        _ => Err(format_err!("cannot ensure endpoint URL for: {}", url)),
    }
}

#[test]
fn test_ensure_endpoint_url() -> Result<(), Error> {
    let url_expected = "https://minecraft.gamepedia.com/api.php".to_string();
    let url_pre = "https://minecraft.gamepedia.com";
    assert_eq!(ensure_endpoint_url(url_pre)?, url_expected);
    let url_pre = "https://minecraft.gamepedia.com/api.php";
    assert_eq!(ensure_endpoint_url(url_pre)?, url_expected);
    Ok(())
}
