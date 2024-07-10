use std::{cmp::Ordering, sync::Arc};

use chrono::Utc;

use crate::{
    infrastructure::{project_repository::ProjectRepository, DbError},
    models::project_model::{Project, ProjectStatus}, User,
};

pub enum ProjectError {
    NotFound,
    TooManyProjects,
    ProjectExistsWithSameName,
    UnknownError,
}

#[derive(Debug)]
pub struct ProjectService {
    repository: Arc<ProjectRepository>,
}

impl ProjectService {
    pub fn new(repository: Arc<ProjectRepository>) -> Self {
        ProjectService { repository }
    }

    pub async fn create(&self, project_name: &str, user: &User) -> Result<Project, ProjectError> {
        // Get existing projects for user
        let existing_projects = match self.repository.get_all(&user.name).await {
            Ok(projects) => projects,
            Err(DbError::NotFound) => return Err(ProjectError::NotFound),
            Err(DbError::FailedConvertion(msg)) => {
                println!("{msg}");
                return Err(ProjectError::UnknownError);
            }
            Err(DbError::UnknownError) => return Err(ProjectError::UnknownError),
        };

        // Each user can maximum have 15 projects
        if existing_projects.len() >= 15 {
            return Err(ProjectError::TooManyProjects);
        }

        // Check if there already is a project with the same name
        if existing_projects.iter().any(|p| p.name == project_name) {
            return Err(ProjectError::ProjectExistsWithSameName);
        }

        // Create an insert the new project into the db
        let project = Project::new(project_name, &user.name);
        match self.repository.insert(project).await {
            Ok(project) => Ok(project),
            Err(_) => Err(ProjectError::UnknownError),
        }
    }

    pub async fn get_all(&self, user: &User) -> Result<Vec<Project>, ProjectError> {
        match self.repository.get_all(&user.name).await {
            Ok(mut projects) => {
                projects.sort_by(|a, b| match (&a.status, &b.status) {
                    (ProjectStatus::ACTIVE, ProjectStatus::INACTIVE) => Ordering::Less,
                    (ProjectStatus::INACTIVE, ProjectStatus::ACTIVE) => Ordering::Greater,
                    _ => {
                        let a_date = a.modified_at.unwrap_or(a.created_at);
                        let b_date = b.modified_at.unwrap_or(b.created_at);
                        b_date.cmp(&a_date)
                    }
                });
                Ok(projects)
            }
            Err(e) => match e {
                DbError::NotFound => Err(ProjectError::NotFound),
                DbError::FailedConvertion(msg) => {
                    println!("{msg}");
                    Err(ProjectError::UnknownError)
                }
                DbError::UnknownError => Err(ProjectError::UnknownError),
            },
        }
    }

    pub async fn get(&self, project_id: &str, user: &User) -> Result<Project, ProjectError> {
        let result = self.repository.get(project_id, &user.name).await;

        match result {
            Ok(project) => Ok(project),
            Err(DbError::NotFound) => Err(ProjectError::NotFound),
            Err(DbError::FailedConvertion(msg)) => {
                println!("{msg}");
                Err(ProjectError::UnknownError)
            }
            Err(DbError::UnknownError) => Err(ProjectError::UnknownError),
        }
    }

    pub async fn update(&self, project: &mut Project, user: &User) -> Result<Project, ProjectError> {
        // Update modified at & by
        project.modified_at = Some(Utc::now());
        project.modified_by = Some(user.name.to_string());

        let result = self.repository.update(project).await;

        match result {
            Ok(project) => Ok(project),
            Err(DbError::NotFound) => Err(ProjectError::NotFound),
            Err(DbError::FailedConvertion(msg)) => {
                println!("{msg}");
                Err(ProjectError::UnknownError)
            }
            Err(DbError::UnknownError) => Err(ProjectError::UnknownError),
        }
    }

    pub async fn delete(&self, project_id: &str, user: &User) -> Result<(), ProjectError> {
        let result = self.repository.delete(project_id, &user.name).await;

        match result {
            Ok(_) => Ok(()),
            Err(DbError::NotFound) => Err(ProjectError::NotFound),
            Err(_) => Err(ProjectError::UnknownError),
        }
    }
}
