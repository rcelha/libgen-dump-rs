use async_trait::async_trait;
use sqlx::any::AnyArguments;
use sqlx::query::Query;
use sqlx::Connection;
use sqlx::Executor;
use sqlx::{Any, Transaction};

use super::RepositoryTransaction;

pub struct SqlxRepositoryTransaction<'a> {
    transaction: Transaction<'a, Any>,
}

type _Query<'a> = Query<'a, Any, AnyArguments<'a>>;

#[async_trait(?Send)]
impl<'a> RepositoryTransaction<_Query<'a>> for SqlxRepositoryTransaction<'a> {
    async fn execute(&mut self, query: _Query<'a>) -> Result<(), ()> {
        self.transaction.begin().await.unwrap();
        self.transaction.execute(query).await.unwrap();
        Ok(())
    }

    async fn commit(self) -> Result<(), ()> {
        self.transaction.commit().await.unwrap();
        Ok(())
    }
}

impl<'a> SqlxRepositoryTransaction<'a> {
    pub fn new(transaction: Transaction<'a, Any>) -> Self {
        SqlxRepositoryTransaction { transaction }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use sqlx::AnyConnection;
    use sqlx::Connection;
    use sqlx::SqliteConnection;

    async fn mk_conn() -> AnyConnection {
        let conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        conn.into()
    }

    #[tokio::test]
    async fn sanity_check() {
        let mut conn = mk_conn().await;
        let transaction = conn.begin().await.unwrap();
        let _t = SqlxRepositoryTransaction::new(transaction);
    }

    #[tokio::test]
    async fn test_queries() {
        let mut conn = mk_conn().await;
        let tables = sqlx::query("SELECT name FROM sqlite_master WHERE type = \"table\"")
            .fetch_all(&mut conn)
            .await
            .unwrap();
        assert_eq!(tables.len(), 0);

        let transaction = conn.begin().await.unwrap();
        let mut t = SqlxRepositoryTransaction::new(transaction);
        let q1 = sqlx::query::<Any>("CREATE TABLE test_queries (id INTEGER PRIMARY KEY)");
        t.execute(q1).await.unwrap();

        let q2 = sqlx::query::<Any>("CREATE TABLE test_queries_2 (id INTEGER PRIMARY KEY)");
        t.execute(q2).await.unwrap();

        t.commit().await.unwrap();

        let tables = sqlx::query("SELECT name FROM sqlite_master WHERE type = \"table\"")
            .fetch_all(&mut conn)
            .await
            .unwrap();
        assert_eq!(tables.len(), 2);
    }
}
