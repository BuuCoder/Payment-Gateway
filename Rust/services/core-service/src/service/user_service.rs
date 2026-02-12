use anyhow::Result;
use crate::domain::user::User;
use crate::repo::user_repo::UserRepository;

#[derive(Clone)]
pub struct UserService {
    repository: UserRepository,
}

impl UserService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        Ok(self.repository.find_all().await?)
    }

    pub async fn get_user_by_id(&self, id: i32) -> Result<User> {
        Ok(self.repository.find_by_id(id).await?)
    }
}
