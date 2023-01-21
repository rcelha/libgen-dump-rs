use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};
use tokio_stream::wrappers::ReadDirStream;

use crate::{
    models::LibgenBook,
    transaction::{
        fs::{FileSystemCommand, FileSystemRepositoryTransaction},
        RepositoryTransaction,
    },
};

use super::LibgenSearchOptions;

pub struct FileSystemRepository {
    basepath: PathBuf,
}

impl FileSystemRepository {
    pub fn new(basepath: &str) -> FileSystemRepository {
        let basepath = basepath.into();
        FileSystemRepository { basepath }
    }

    fn is_extension_valid(&self, book: &LibgenBook) -> bool {
        match book.extension.as_str() {
            "zip" | "cbz" | "gz" | "html" | "lit" | "txt" | "cbr" | "docx" | "chm" | "rtf"
            | "fb2" | "azw3" | "mobi" | "doc" | "djvu" | "epub" | "pdf" => true,
            _ => false,
        }
    }
}

#[async_trait(?Send)]
impl super::LibgenRepository for FileSystemRepository {
    type Error = std::io::Error;
    type Query = FileSystemCommand;
    type Transaction = FileSystemRepositoryTransaction;

    async fn initialize_repository(&mut self) {
        tokio::fs::create_dir_all(&self.basepath).await.unwrap();
    }

    /// It only supports `LibgenSearchOptions.match_any` for now
    async fn search(
        &mut self,
        options: LibgenSearchOptions,
    ) -> BoxStream<Result<LibgenBook, std::io::Error>> {
        let read_dir = tokio::fs::read_dir(&self.basepath).await.unwrap();
        let mut stream_read_dir = ReadDirStream::new(read_dir);

        let stream = async_stream::stream! {
            while let Some(Ok(dir_entry)) = stream_read_dir.next().await {
                let file_name = dir_entry.file_name().into_string().unwrap();

                if let Some(ref search_value) = options.match_any {
                    let matches = file_name.matches(search_value);
                    if matches.count() == 0 {
                        continue;
                    }
                }

                let file_name2 = file_name.clone();
                let mut split_name = file_name2.rsplitn(2, ".");
                let extension = split_name.next().unwrap_or("").to_string();
                let title = split_name.next().unwrap_or("").to_string();

                let book = LibgenBook {
                    md5: "".to_string(),
                    title,
                    extension,
                    author: "".to_string(),
                    ipfs_cid: None,
                    path: Some(file_name.clone()),
                    content: None,
                    language: "".to_string(),
                };
                if !self.is_extension_valid(&book) {
                    continue;
                }

                let fullpath = dir_entry.path();
                let enriched_book = enrich_book_from_xattrs(fullpath, book);
                yield Ok(enriched_book);
            }
        };
        stream.boxed()
    }

    async fn insert_book(&mut self, transaction: &mut Self::Transaction, book: LibgenBook) {
        let file_name = format!("{}", book);
        let mut path = self.basepath.to_path_buf();
        path.push(file_name);
        let path = path.to_string_lossy().to_string();

        let content = book.content.as_ref().unwrap().clone();
        let xattrs = build_xattrs_from_book(&book);

        let insert_command = FileSystemCommand::INSERT(path, content, xattrs);
        transaction.execute(insert_command).await.ok();
    }

    async fn get_total(&mut self) -> usize {
        self.list_books().await.count().await
    }
}

fn xattr_get<N, P>(path: P, name: N) -> String
where
    P: AsRef<Path>,
    N: AsRef<OsStr>,
{
    let x = xattr::get(&path, name).unwrap_or(None);
    let z = x
        .as_ref()
        .map(|i| {
            let i = std::str::from_utf8(i).unwrap_or("");
            i.to_string()
        })
        .unwrap_or_default();
    z
}

fn enrich_book_from_xattrs<P>(path: P, mut book: LibgenBook) -> LibgenBook
where
    P: AsRef<Path>,
{
    let md5 = xattr_get(&path, "user.libgen-md5");
    let title = xattr_get(&path, "user.libgen-title");
    let author = xattr_get(&path, "user.libgen-author");
    let ipfs_cid = xattr_get(&path, "user.libgen-ipfs_cid");
    let language = xattr_get(&path, "user.libgen-language");

    book.md5 = md5;
    book.title = title;
    book.author = author;
    if !ipfs_cid.is_empty() {
        book.ipfs_cid = Some(ipfs_cid);
    };
    book.language = language;

    book
}

fn build_xattrs_from_book(book: &LibgenBook) -> HashMap<String, String> {
    let mut book_xattrs = HashMap::new();

    book_xattrs.insert("user.libgen-md5".to_string(), book.md5.clone());
    book_xattrs.insert("user.libgen-title".to_string(), book.title.clone());
    book_xattrs.insert("user.libgen-author".to_string(), book.author.clone());
    if let Some(ref ipfs_cid) = book.ipfs_cid {
        book_xattrs.insert("user.libgen-ipfs_cid".to_string(), ipfs_cid.to_string());
    };
    book_xattrs.insert("user.libgen-language".to_string(), book.language.clone());
    book_xattrs
}

#[cfg(test)]
mod test {
    use crate::repositories::LibgenRepository;

    use super::*;

    #[tokio::test]
    async fn sanity_check() {
        let mut repos = FileSystemRepository::new("/home/rcelha/tmp");

        let book = LibgenBook {
            md5: "12345".to_string(),
            title: "The lord of the rings".to_string(),
            extension: "epub".to_string(),
            author: "Tokien".to_string(),
            ipfs_cid: None,
            path: None,
            content: Some(b"The Lord of the Rings".to_vec()),
            language: "English".to_string(),
        };

        let mut t = FileSystemRepositoryTransaction::new();
        repos.insert_book(&mut t, book).await;
        t.commit().await.unwrap();

        let mut result = repos.list_books().await;

        while let Some(i) = result.next().await {
            println!("{:?}", i);
        }
    }
}
