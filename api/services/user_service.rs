use std::sync::Arc;

use crate::{
    infrastructure::{
        database::DbError,
        user_repository::UserRepository,
    },
    models::user_model::{User, UserRole},
};

use super::project_service::{ProjectError, ProjectService};

#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    #[error("User does have the required permissions to perform this action")]
    Forbidden,
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("Project error")]
    ProjectError(ProjectError),
}

impl From<DbError> for UserError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound => UserError::NotFound,
            DbError::Unknown(err) => UserError::Unknown(err),
            _ => UserError::Unknown(String::from("Something went wrong")),
        }
    }
}

impl From<ProjectError> for UserError {
    fn from(error: ProjectError) -> Self {
        UserError::ProjectError(error)
    }
}

#[derive(Debug)]
pub struct UserService {
    pub repository: Arc<UserRepository>,
    pub project_service: Arc<ProjectService>,
}

impl UserService {
    pub fn new(repository: Arc<UserRepository>, project_service: Arc<ProjectService>) -> Self {
        UserService {
            repository,
            project_service,
        }
    }

    pub async fn create(
        &self,
        name: &str,
        role: &UserRole,
        created_by: &User,
    ) -> Result<User, UserError> {
        let user = User::new(name, role, &created_by.id);
        self.repository.create(&user).await?;

        Ok(user)
    }

    pub async fn get_by_api_key(&self, api_key: &str) -> Result<User, UserError> {
        let user = self.repository.get_by_api_key(api_key).await?;
        Ok(user)
    }

    pub async fn get_by_id(&self, user_id: &str, include_api_key: bool) -> Result<User, UserError> {
        let mut user = self.repository.get_by_id(user_id).await?;

        if !include_api_key {
            user.api_key = None;
        }

        Ok(user)
    }

    pub async fn get_all(&self) -> Result<Vec<User>, UserError> {
        let mut users = self.repository.get_all().await?;

        // Do not return the API KEY when listing users
        users.iter_mut().for_each(|user| user.api_key = None);

        Ok(users)
    }

    pub async fn delete(&self, user_id: &str) -> Result<(), UserError> {
        // Delete all projects of the user
        let user = self.get_by_id(user_id, true).await?;

        // Delete all projects by user
        let projects = self.project_service.get_all(&user).await?;
        for project in projects {
            self.project_service.delete(&user, &project.id).await?;
        }

        // Delete user
        self.repository.delete(&user).await?;

        Ok(())
    }
}
