use crate::application::queries::postgres::UserQueries;
use crate::infrastructure::error::DbError;
use crate::infrastructure::postgres::rows::user::User;

impl UserQueries {
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, DbError> {
        self.repo.get_user_by_username(username).await
    }
}
