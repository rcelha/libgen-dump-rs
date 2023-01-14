#[cfg(feature = "cli")]
use clap::Parser;
use futures::StreamExt;
use libgen_dump_rs::repositories::*;
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
async fn origin_repos(conn: String) -> impl LibgenRepository {
    println!("trying to connect to {}", conn);
    let conn = MySqlConnection::connect(&conn).await.unwrap();
    MysqlLibgenRepository { conn }
}

#[cfg(feature = "cli")]
async fn target_repos(path: String) -> impl LibgenRepository {
    let url = format!("sqlite://{}?mode=rwc", path);
    let conn = SqliteConnection::connect(&url).await.unwrap();
    SqliteTargetRepository { conn }
}

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() {
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
    while let Some((idx, Ok(i))) = books_stream.next().await {
        if idx % step == 0 {
            println!("{}%", idx / step);
        }
        sqlite.insert_book(i).await;
    }
}

#[cfg(not(feature = "cli"))]
fn main() {}
