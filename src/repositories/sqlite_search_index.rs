use std::marker::PhantomData;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use sqlx::any::AnyArguments;
use sqlx::query::Query;
use sqlx::sqlite::SqliteConnection;
use sqlx::Any;
use sqlx::QueryBuilder;
use sqlx::Sqlite;
use sqlx::{Connection, Row};

use crate::models::LibgenBook;
use crate::transaction::sqlx::SqlxRepositoryTransaction;
use crate::transaction::RepositoryTransaction;

use super::AttributeSort;
use super::LibgenSearchOptions;

pub struct SqliteTargetRepository<'a> {
    pub conn: SqliteConnection,
    phantom_data: PhantomData<&'a ()>,
}

impl<'a> SqliteTargetRepository<'a> {
    pub fn new(conn: SqliteConnection) -> SqliteTargetRepository<'a> {
        SqliteTargetRepository {
            conn,
            phantom_data: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<'a> super::LibgenRepository for SqliteTargetRepository<'a> {
    type Error = sqlx::Error;
    type Query = Query<'a, Any, AnyArguments<'a>>;
    type Transaction = SqlxRepositoryTransaction<'a>;

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

    async fn search(
        &mut self,
        options: LibgenSearchOptions,
    ) -> BoxStream<Result<LibgenBook, Self::Error>> {
        let mut query_builder = QueryBuilder::<Sqlite>::new(
            r#"SELECT
                   md5,
                   title,
                   extension,
                   author,
                   ipfs_cid,
                   language
                FROM libgen
                WHERE 1
            "#,
        );
        if let Some(x) = options.match_any.as_ref() {
            query_builder.push("AND libgen MATCH");
            query_builder.push_bind(x.clone());
        }

        match options.sort {
            Some((AttributeSort::RANK, direction)) => {
                query_builder.push(format!("ORDER BY rank {:?}", direction));
            }
            _ => (),
        };

        let stream = async_stream::stream! {
            let q = query_builder.build();
            let mut result = q.fetch(&mut self.conn);

            while let Some(Ok(row)) = result.next().await {
                let md5 = row.get("md5");
                let title = row.get("title");
                let extension = row.get("extension");
                let author = row.get("author");
                let ipfs_cid = Some(row.get("ipfs_cid"));
                let path = None;
                let content = None;
                let language = row.get("language");

                let book = LibgenBook {
                    md5,
                    title,
                    extension,
                    author,
                    ipfs_cid,
                    path,
                    content,
                    language,
                };
                yield Ok(book);
            }
        };
        stream.boxed()
    }

    async fn get_total(&mut self) -> usize {
        let q = sqlx::query(r#"SELECT count(*) as total FROM libgen"#);
        let row = q.fetch_one(&mut self.conn).await.unwrap();
        let total: i64 = row.get("total");
        total as usize
    }

    async fn insert_book(&mut self, transaction: &mut Self::Transaction, book: LibgenBook) {
        let q = sqlx::query(
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
        .bind(book.language);
        transaction.execute(q).await.unwrap();
    }
}
