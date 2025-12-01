use tokio_postgres::Row;

pub struct User {
    pub username: String,
    pub hashed_password: String,
}

impl From<Row> for User {
    fn from(row: Row) -> Self {
        Self {
            username: row.get("username"),
            hashed_password: row.get("hashed_password"),
        }
    }
}