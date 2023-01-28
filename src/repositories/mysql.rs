use std::marker::PhantomData;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use futures::TryStreamExt;
use sqlx::any::AnyArguments;
use sqlx::mysql::MySqlConnection;
use sqlx::query::Query;
use sqlx::Any;
use sqlx::Row;

use crate::models::LibgenBook;
use crate::transaction::sqlx::SqlxRepositoryTransaction;

use super::LibgenSearchOptions;

pub struct MysqlLibgenRepository<'a> {
    pub conn: MySqlConnection,
    phantom_data: PhantomData<&'a ()>,
}

impl<'a> MysqlLibgenRepository<'a> {
    pub fn new(conn: MySqlConnection) -> MysqlLibgenRepository<'a> {
        MysqlLibgenRepository {
            conn,
            phantom_data: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<'a> super::LibgenRepository for MysqlLibgenRepository<'a> {
    type Error = sqlx::Error;
    type Query = Query<'a, Any, AnyArguments<'a>>;
    type Transaction = SqlxRepositoryTransaction<'a>;

    /// Does not support any LibgenSearchOptions options for now
    async fn search(
        &mut self,
        _options: LibgenSearchOptions,
    ) -> BoxStream<Result<LibgenBook, Self::Error>> {
        let sql = r#"
               SELECT
                   u.MD5, u.Title, u.Extension, u.Author, u.Language, h.ipfs_cid
               FROM updated as u
               INNER JOIN hashes as h ON u.MD5 = h.MD5
            "#;
        let q = sqlx::query(sql);

        q.fetch(&mut self.conn)
            .map_ok(|row| {
                let md5 = row.get("MD5");
                let title = row.get("Title");
                let file_extension = row.get("Extension");
                let author = row.get("Author");
                let ipfs_cid = Some(row.get("ipfs_cid"));
                let path = None;
                let content = None;
                let language = row.get("Language");

                LibgenBook {
                    md5,
                    title,
                    file_extension,
                    author,
                    ipfs_cid,
                    path,
                    content,
                    language,
                }
            })
            .boxed()
    }

    async fn get_total(&mut self) -> usize {
        let q = sqlx::query(r#"SELECT count(*) as total FROM updated"#);
        let row = q.fetch_one(&mut self.conn).await.unwrap();
        let total: i64 = row.get("total");
        total as usize
    }

    async fn insert_book(&mut self, _transaction: &mut Self::Transaction, _book: LibgenBook) {
        todo!();
    }
}
