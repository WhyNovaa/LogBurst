use axum::response::Response;
use sqlx::Database;
use tokio::sync::oneshot;
use crate::models::log::Log;
use crate::traits::client::Client;
use crate::traits::data_base::DataBase;
use crate::traits::start::Start;

/*pub struct App<C, D>
where
    C: Client,
    D: Start + DataBase,
{
    http_client: C,
    db: D,
}

impl<C, D> App<C, D>
where
    C: Client,
    D: Start + DataBase,
{
    pub async fn new() -> Self {
        let (s, r) = tokio::sync::mpsc::channel::<(Log, oneshot::Sender<Response>)>(100);

        let http_client = C::new(s);
        let db = <D as Database>::new(r);

        Self {
            http_client,
            db,
        }
    }

    pub fn start(self) {

    }
}*/
