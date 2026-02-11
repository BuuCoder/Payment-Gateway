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

    pub async fn get_all_users(&self) -> Result<Vec<User>, String> {
        self.repository
            .find_all()
            .await
            .map_err(|e| format!("Failed to fetch users: {}", e))
    }

    pub async fn get_user_by_id(&self, id: i32) -> Result<User, String> {
        self.repository
            .find_by_id(id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => "User not found".to_string(),
                _ => format!("Database error: {}", e),
            })
    }
}
