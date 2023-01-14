use std::fmt::Debug;

use futures::stream::BoxStream;
use futures::StreamExt;
use futures::TryStreamExt;
use sqlx::mysql::MySqlConnection;
use sqlx::sqlite::SqliteConnection;
use sqlx::{Connection, Row};

use crate::models::LibgenBook;

#[async_trait::async_trait]
pub trait LibgenRepository {
    type Error: Debug;

    async fn initialize_repository(&mut self) {}

    async fn list_books(&mut self) -> BoxStream<Result<LibgenBook, Self::Error>>;

    async fn search(&mut self, value: String) -> BoxStream<Result<LibgenBook, Self::Error>>;

    async fn insert_book(&mut self, book: LibgenBook);

    async fn get_total(&mut self) -> usize;
}

pub struct SqliteTargetRepository {
    pub conn: SqliteConnection,
}

#[async_trait::async_trait]
impl LibgenRepository for SqliteTargetRepository {
    type Error = sqlx::Error;

    async fn initialize_repository(&mut self) {
        let mut transaction = self.conn.begin().await.unwrap();
        sqlx::query(
            r#"CREATE VIRTUAL TABLE IF NOT EXISTS libgen
               USING FTS5(
                   md5 UNINDEXED,
                   title,
                   extension,
                   author,
                   ipfs_cid UNINDEXED,
                   language,
               )"#,
        )
        .execute(&mut transaction)
        .await
        .unwrap();
        transaction.commit().await.unwrap();
    }

    async fn list_books(&mut self) -> BoxStream<Result<LibgenBook, Self::Error>> {
        todo!();
    }

    async fn search(&mut self, value: String) -> BoxStream<Result<LibgenBook, Self::Error>> {
        let q = sqlx::query(
            r#"SELECT
                   md5,
                   title,
                   extension,
                   author,
                   ipfs_cid,
                   language
                FROM libgen
                WHERE libgen MATCH $1
                ORDER BY rank DESC
            "#,
        )
        .bind(value);
        q.fetch(&mut self.conn)
            .map_ok(|row| {
                let md5 = row.get("md5");
                let title = row.get("title");
                let extension = row.get("extension");
                let author = row.get("author");
                let ipfs_cid = row.get("ipfs_cid");
                let language = row.get("language");

                LibgenBook {
                    md5,
                    title,
                    extension,
                    author,
                    ipfs_cid,
                    language,
                }
            })
            .boxed()
    }

    async fn get_total(&mut self) -> usize {
        todo!();
    }

    async fn insert_book(&mut self, book: LibgenBook) {
        let mut transaction = self.conn.begin().await.unwrap();
        sqlx::query(
            r#"INSERT INTO
               libgen(md5, title, extension, author, ipfs_cid, language)
               VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(book.md5)
        .bind(book.title)
        .bind(book.extension)
        .bind(book.author)
        .bind(book.ipfs_cid)
        .bind(book.language)
        .execute(&mut transaction)
        .await
        .unwrap();
        transaction.commit().await.unwrap();
    }
}

pub struct MysqlLibgenRepository {
    pub conn: MySqlConnection,
}

#[async_trait::async_trait]
impl LibgenRepository for MysqlLibgenRepository {
    type Error = sqlx::Error;

    async fn list_books(&mut self) -> BoxStream<Result<LibgenBook, sqlx::Error>> {
        let q = sqlx::query(
            r#"SELECT
                   u.MD5, u.Title, u.Extension, u.Author, u.Language, h.ipfs_cid
               FROM updated as u
               INNER JOIN hashes as h ON u.MD5 = h.MD5
            "#,
        );
        q.fetch(&mut self.conn)
            .map_ok(|row| {
                let md5 = row.get("MD5");
                let title = row.get("Title");
                let extension = row.get("Extension");
                let author = row.get("Author");
                let ipfs_cid = row.get("ipfs_cid");
                let language = row.get("Language");

                LibgenBook {
                    md5,
                    title,
                    extension,
                    author,
                    ipfs_cid,
                    language,
                }
            })
            .boxed()
    }

    async fn search(&mut self, value: String) -> BoxStream<Result<LibgenBook, Self::Error>> {
        todo!();
    }

    async fn get_total(&mut self) -> usize {
        let q = sqlx::query(r#"SELECT count(*) as total FROM updated"#);
        let row = q.fetch_one(&mut self.conn).await.unwrap();
        let total: i64 = row.get("total");
        total as usize
    }

    async fn insert_book(&mut self, _book: LibgenBook) {
        todo!();
    }
}
