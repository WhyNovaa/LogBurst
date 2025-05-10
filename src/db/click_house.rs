use std::marker::PhantomData;
use async_trait::async_trait;
use crate::traits::logs_repository::LogsRepository;
use crate::traits::start::Start;

pub struct ClickHousePool {
    smth: i32,
}

#[async_trait]
impl Start for ClickHousePool {
    async fn start(self) {
        todo!()
    }
}

impl LogsRepository for ClickHousePool {
    fn new() -> Self {
        Self { smth: 32 }
    }
}