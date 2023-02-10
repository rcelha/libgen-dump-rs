#[cfg(feature = "cli")]
use clap::Parser;
use futures::StreamExt;
use libgen_dump_rs::repositories::*;
use libgen_dump_rs::transaction::sqlx::SqlxRepositoryTransaction;
use libgen_dump_rs::transaction::RepositoryTransaction;
use sqlx::mysql::MySqlConnection;
use sqlx::sqlite::SqliteConnection;
use sqlx::Connection;
use std::path::PathBuf;

/// First line
///
/// Third line
/// And the last line
#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
struct Args {
    /// The MySQL origin connection string
    mysql_conn_string: String,

    /// sqlite output file
    output: PathBuf,
}

#[cfg(feature = "cli")]
async fn origin_repos<'a>(conn: String) -> MysqlLibgenRepository<'a> {
    println!("trying to connect to {}", conn);
    let conn = MySqlConnection::connect(&conn).await.unwrap();
    MysqlLibgenRepository::new(conn)
}

#[cfg(feature = "cli")]
async fn target_conn(path: String) -> SqliteConnection {
    let url = format!("sqlite://{}?mode=rwc", path);
    SqliteConnection::connect(&url).await.unwrap()
}

#[cfg(feature = "cli")]
async fn target_repos<'a>(path: String) -> SqliteTargetRepository<'a> {
    let conn = target_conn(path).await;
    SqliteTargetRepository::new(conn)
}

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() {
    use sqlx::AnyConnection;

    let args = Args::parse();
    println!("{:#?}", args);

    let mut sqlite = target_repos(args.output.to_string_lossy().to_string()).await;
    sqlite.initialize_repository().await;

    let mut mysql = origin_repos(args.mysql_conn_string.clone()).await;
    println!("fetching stuff");

    let total = mysql.get_total().await;
    let step = total / 100;
    let mut books_stream = mysql.list_books().await.enumerate();

    println!("Inserting new books ({} total)", total);

    let tconn = target_conn(args.output.to_string_lossy().to_string()).await;
    let mut conn: AnyConnection = tconn.into();
    let transaction = conn.begin().await.unwrap();
    let mut repos_transaction = SqlxRepositoryTransaction::new(transaction);

    while let Some((idx, Ok(i))) = books_stream.next().await {
        if idx % step == 0 {
            println!("{}%", idx / step);
        }
        sqlite.insert_book(&mut repos_transaction, i).await;
    }
    repos_transaction.commit().await.unwrap();
}

#[cfg(not(feature = "cli"))]
fn main() {}
