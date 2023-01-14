#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct LibgenBook {
    pub md5: String,
    pub title: String,
    pub extension: String,
    pub author: String,
    pub ipfs_cid: String,
    pub language: String,
}
