use crate::traits::data_base::DataBase;
use crate::traits::start::Start;

pub struct App<C, D>
where C: Start,
    D: Start + DataBase,
{
    client: C,
    db: D,
}

impl<C, D> App<C, D> {
    pub async fn new() -> Self {
        let client = C::new();
        let db = D::new();
        Self {
            client,
            db,
        }
    }

    pub fn start(self) {

    }
}
