use async_trait::async_trait;

#[async_trait]
pub trait Get {
  async fn get_by_id(id: Box<[u8]>) -> Self;
}