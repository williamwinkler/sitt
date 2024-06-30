use std::{cmp::Ordering, sync::Arc};

use crate::{
    infrastructure::project_repository::ProjectRepository,
    models::project_model::{Project, ProjectStatus},
};

#[derive(Debug)]
pub struct ProjectService {
    repository: Arc<ProjectRepository>,
}

static USER: &str = "admin";

impl ProjectService {
    pub fn new(repository: Arc<ProjectRepository>) -> Self {
        ProjectService { repository }
    }

    pub async fn create(&self, project_name: &str) -> Result<Project, String> {
        let existing_projects = self.repository.get_all(USER).await.unwrap();

        if existing_projects.len() >= 15 {
            return Err("Too many projects. Cannot create a new one".to_string());
        }
        if existing_projects.iter().any(|p| p.name == project_name) {
            return Err("A project with the same name already exists".to_string());
        }

        let project = Project::new(project_name, USER);
        let result = self.repository.insert(project).await;
        match result {
            Ok(project) => Ok(project),
            Err(msg) => Err(msg),
        }
    }

    pub async fn get_all(&self, created_by: &str) -> Result<Vec<Project>, String> {
        self.repository
            .get_all(USER)
            .await
            .map(|mut projects| {
                projects.sort_by(|a, b| match (&a.status, &b.status) {
                    (ProjectStatus::ACTIVE, ProjectStatus::INACTIVE) => Ordering::Less,
                    (ProjectStatus::INACTIVE, ProjectStatus::ACTIVE) => Ordering::Greater,
                    _ => {
                        let a_date = a.modified_at.unwrap_or(a.created_at);
                        let b_date = b.modified_at.unwrap_or(b.created_at);
                        b_date.cmp(&a_date)
                    }
                });
                projects
            })
            .map_err(|e| e.to_string())
    }

    pub async fn get(&self, project_id: &str, created_by: &str) -> Result<Project, String> {
        return self.repository.get(project_id, created_by).await;
    }

    pub async fn delete(&self, project_id: &str, created_by: &str) -> Result<(), String> {
        return self.repository.delete(project_id, created_by).await;
    }
}

// pub async fn update_project(
//     project_name: &str,
//     update_project: UpdateProjectDto,
// ) -> Result<ProjectDto, String> {
//     // Implement logic to update an existing project in the database
//     Ok(ProjectDto {
//         name: update_project.name,
//         status: Status::INACTIVE,
//         created_at: "2021-01-01T00:00:00Z".into(),
//         modified_at: "2021-01-01T00:00:00Z".into(),
//     })
// }
