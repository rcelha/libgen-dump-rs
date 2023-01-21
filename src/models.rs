use std::fmt::{Display, Write};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct LibgenBook {
    pub md5: String,
    pub title: String,
    pub extension: String,
    pub author: String,
    pub ipfs_cid: Option<String>,
    pub path: Option<String>,
    pub content: Option<Vec<u8>>,
    pub language: String,
}

impl Display for LibgenBook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.md5)?;
        f.write_char('-')?;
        f.write_str(&self.author)?;
        f.write_char('-')?;
        f.write_str(&self.title)?;
        f.write_char('-')?;
        f.write_str(&self.language)?;
        f.write_char('.')?;
        f.write_str(&self.extension)?;
        Ok(())
    }
}
