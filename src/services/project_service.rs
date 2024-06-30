use std::sync::Arc;

use crate::{
    infrastructure::project_repository::ProjectRepository, models::project_model::Project,
};

#[derive(Debug)]
pub struct ProjectService {
    repository: Arc<ProjectRepository>,
}

impl ProjectService {
    pub fn new(repository: Arc<ProjectRepository>) -> Self {
        ProjectService { repository }
    }

    pub async fn create(&self, project_name: &str) -> Result<Project, String> {
        let project = Project::new(project_name, "admin");
        let result = self.repository.insert(project).await;
        match result {
            Ok(project) => Ok(project),
            Err(msg) => Err(msg),
        }
    }

    pub async fn get_all(&self) -> Result<Vec<Project>, String> {
        return self.repository.get_all("admin".to_string()).await;
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

// pub async fn delete_project(project_name: &str) -> Result<(), String> {
//     // Implement logic to delete a project from the database
//     Ok(())
// }
