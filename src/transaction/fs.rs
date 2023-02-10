use std::{collections::HashMap, fmt::Debug, path::Path};

use async_trait::async_trait;

pub struct FileSystemRepositoryTransaction {
    files: Vec<(String, Vec<u8>, HashMap<String, String>)>,
}

impl FileSystemRepositoryTransaction {
    pub fn new() -> FileSystemRepositoryTransaction {
        FileSystemRepositoryTransaction { files: vec![] }
    }
}

pub enum FileSystemCommand {
    // INSERT(path, content, xattrs
    INSERT(String, Vec<u8>, HashMap<String, String>),
}

#[async_trait(?Send)]
impl super::RepositoryTransaction<FileSystemCommand> for FileSystemRepositoryTransaction {
    async fn execute(&mut self, query: FileSystemCommand) -> Result<(), ()> {
        match query {
            FileSystemCommand::INSERT(path, content, xattrs) => {
                self.files.push((path, content, xattrs));
            }
        };
        Ok(())
    }

    // TODO write to tmp first and move on commit
    async fn commit(mut self) -> Result<(), ()> {
        while let Some(i) = self.files.pop() {
            let fname = i.0;
            let contents = i.1;
            let xattrs = i.2;
            let path = Path::new(&fname);
            std::fs::write(&path, contents).unwrap();
            xattr_bulk_apply(&path, xattrs);
        }
        Ok(())
    }
}

fn xattr_bulk_apply<P>(path: P, xattrs: HashMap<String, String>)
where
    P: AsRef<Path> + Clone + Debug,
{
    for (k, v) in &xattrs {
        xattr::set(path.clone(), k, v.as_bytes()).unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::transaction::RepositoryTransaction;

    #[tokio::test]
    async fn sanity_check() {
        let mut t = FileSystemRepositoryTransaction::new();
        t.execute(FileSystemCommand::INSERT(
            "/tmp/file1.txt".to_string(),
            "test".as_bytes().into(),
            HashMap::new(),
        ))
        .await
        .unwrap();
        t.execute(FileSystemCommand::INSERT(
            "/tmp/file2.txt".to_string(),
            "test 2".as_bytes().into(),
            HashMap::new(),
        ))
        .await
        .unwrap();
        t.commit().await.unwrap();
    }
}
