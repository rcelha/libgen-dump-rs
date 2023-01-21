use std::fmt::Debug;

use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::models::LibgenBook;
use crate::transaction::RepositoryTransaction;

mod sqlite_search_index;
pub use sqlite_search_index::*;

mod mysql;
pub use mysql::*;

mod fs;
pub use fs::*;

#[async_trait(?Send)]
pub trait LibgenRepository {
    type Error: Debug;
    type Query;
    type Transaction: RepositoryTransaction<Self::Query>;

    async fn initialize_repository(&mut self) {}

    async fn list_books(&mut self) -> BoxStream<Result<LibgenBook, Self::Error>> {
        self.search(Default::default()).await
    }

    async fn search(
        &mut self,
        options: LibgenSearchOptions,
    ) -> BoxStream<Result<LibgenBook, Self::Error>>;

    async fn insert_book(&mut self, transaction: &mut Self::Transaction, book: LibgenBook);

    async fn get_total(&mut self) -> usize;
}

#[derive(Debug)]
pub enum Sort {
    ASC,
    DESC,
}

#[derive(Debug)]
pub enum AttributeSort {
    RANK,
    TITLE,
}

#[derive(Default, Debug)]
pub struct LibgenSearchOptions {
    pub match_any: Option<String>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub sort: Option<(AttributeSort, Sort)>,
}
