use async_trait::async_trait;

#[async_trait]
pub trait Start {
    async fn start(self);
}