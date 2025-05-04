use async_trait::async_trait;
use crate::traits::new::New;

pub trait Start {
    async fn start(self);
}