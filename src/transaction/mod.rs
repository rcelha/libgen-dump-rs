use async_trait::async_trait;

pub mod sqlx;
pub mod fs;

#[async_trait(?Send)]
pub trait RepositoryTransaction<T> {
    async fn execute(&mut self, query: T) -> Result<(), ()>;
    async fn commit(self) -> Result<(), ()>;
}
