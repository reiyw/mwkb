use failure::Error;
use url::Url;

pub fn ensure_endpoint_api_url(url: &str) -> Result<String, Error> {
    let parsed = Url::parse(url)?;
    Ok(format!("{}://{}/api.php", parsed.scheme(), parsed.host_str().unwrap()))
}

pub fn ensure_endpoint_index_url(url: &str) -> Result<String, Error> {
    let parsed = Url::parse(url)?;
    Ok(format!("{}://{}/index.php", parsed.scheme(), parsed.host_str().unwrap()))
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
}
