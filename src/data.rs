use std::collections::HashSet;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use failure::Error;
use glob::{glob, Paths, PatternError};

use parser::Doc;
use title::{load_titles, save_titles, Title};

/// データ関連のパスを管理する
///
/// 何でもかんでもこいつを通せば正しい場所から／にファイルを読み書きできる．
pub struct Data {
    pub base_dir: PathBuf,
    pub title_file: PathBuf,
    pub markuped_text_dir: PathBuf,
    pub parsed_text_dir: PathBuf,
    pub biluo_dir: PathBuf,
    markuped_text_file_extension: &'static str,
    parsed_text_file_extension: &'static str,
    biluo_file_extension: &'static str,
}

impl Data {
    pub fn new(base_dir: &str) -> Data {
        let base_dir = Path::new(base_dir);
        let title_file = base_dir.join("titles.csv");
        let markuped_text_dir = base_dir.join("raw");
        let parsed_text_dir = base_dir.join("parsed");
        let biluo_dir = base_dir.join("biluo");

        if !base_dir.exists() {
            fs::create_dir(&base_dir).unwrap();
        }
        if !markuped_text_dir.exists() {
            fs::create_dir(&markuped_text_dir).unwrap();
        }
        if !parsed_text_dir.exists() {
            fs::create_dir(&parsed_text_dir).unwrap();
        }
        if !biluo_dir.exists() {
            fs::create_dir(&biluo_dir).unwrap();
        }

        Data {
            base_dir: base_dir.to_path_buf(),
            title_file,
            markuped_text_dir,
            parsed_text_dir,
            biluo_dir,
            markuped_text_file_extension: "txt",
            parsed_text_file_extension: "json",
            biluo_file_extension: "tsv",
        }
    }

    pub fn markuped_text_files(&self) -> Result<Paths, PatternError> {
        let pattern = format!(
            "{}/*.{}",
            self.markuped_text_dir.to_str().unwrap(),
            self.markuped_text_file_extension
        );
        glob(&pattern[..])
    }

    fn parsed_text_files(&self) -> Result<Paths, PatternError> {
        let pattern = format!(
            "{}/*.{}",
            self.parsed_text_dir.to_str().unwrap(),
            self.parsed_text_file_extension
        );
        glob(&pattern[..])
    }

    fn biluo_files(&self) -> Result<Paths, PatternError> {
        let pattern = format!(
            "{}/*.{}",
            self.biluo_dir.to_str().unwrap(),
            self.biluo_file_extension
        );
        glob(&pattern[..])
    }

    pub fn make_pageid_set_from_markuped_text_files(&self) -> Result<HashSet<u32>, Error> {
        let mut ids = HashSet::new();
        for entry in self.markuped_text_files()? {
            match entry {
                Ok(path) => {
                    let stem = path.file_stem().unwrap();
                    let id: u32 = stem.to_str().unwrap().parse()?;
                    ids.insert(id);
                }
                Err(e) => {
                    format_err!("{:?}", e);
                }
            }
        }
        Ok(ids)
    }

    pub fn load_titles(&self) -> Result<Vec<Title>, Error> {
        load_titles(&self.title_file)
    }

    pub fn save_titles(&self, titles: &Vec<Title>) -> Result<(), Error> {
        save_titles(titles, &self.title_file)
    }

    pub fn save_markuped_text(&self, pageid: u32, text: &str) -> std::io::Result<()> {
        let filename = format!("{}.{}", pageid, self.markuped_text_file_extension);
        let filepath = self.markuped_text_dir.join(&filename[..]);
        let mut f = fs::File::create(filepath)?;
        f.write_all(text.as_bytes())?;
        Ok(())
    }

    pub fn save_parsed_text(&self, pageid: u32, doc: &Doc) -> Result<(), Error> {
        let filename = format!("{}.{}", pageid, self.parsed_text_file_extension);
        let filepath = self.parsed_text_dir.join(&filename[..]);
        let mut f = fs::File::create(filepath)?;
        f.write_all(serde_json::to_string(doc)?.as_bytes())?;
        Ok(())
    }
}

pub fn parse_pageid(path: &Path) -> u32 {
    let stem = path.file_stem().unwrap();
    stem.to_str().unwrap().parse().unwrap()
}

#[cfg(test)]
mod tests {
    use failure::Error;

    use super::*;

    #[test]
    fn test_datapath() -> Result<(), Error> {
        let data = Data::new("test_dir_1145141919810");
        assert_eq!(
            data.title_file.to_str(),
            Some("test_dir_1145141919810/titles.csv")
        );

        assert!(data.markuped_text_dir.exists());

        let ids = data.make_pageid_set_from_markuped_text_files()?;
        assert_eq!(ids.len(), 0);

        fs::File::create("test_dir_1145141919810/raw/893.txt")?;
        let ids = data.make_pageid_set_from_markuped_text_files()?;
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&893));

        fs::remove_dir_all(data.base_dir)?;
        Ok(())
    }
}
