use crate::{
    models::project_model::{Project, ProjectStatus},
    services::project_service::ProjectError,
};
use chrono::{DateTime, Utc};

use super::{database::Database, DbError};
use aws_sdk_dynamodb::types::{
    AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ScalarAttributeType,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct ProjectRepository {
    db: Arc<Database>,
}

static TABLE_NAME: &str = "projects";

impl ProjectRepository {
    pub async fn new(db: Arc<Database>) -> Self {
        // Partion key: created_by
        let attr_part = AttributeDefinition::builder()
            .attribute_name("created_by")
            .attribute_type(ScalarAttributeType::S)
            .build()
            .expect("Error building the attribute partion 'created_by' in the 'projects' table");

        let keyschema_part = KeySchemaElement::builder()
            .attribute_name("created_by")
            .key_type(KeyType::Hash)
            .build()
            .expect("Error building the key schema partion");

        // Sort Key: id
        let attr_sort = AttributeDefinition::builder()
            .attribute_name("id")
            .attribute_type(ScalarAttributeType::S)
            .build()
            .expect("Error building the attribute partion 'id' in the 'projects' table");

        let keyschema_sort = KeySchemaElement::builder()
            .attribute_name("id")
            .key_type(KeyType::Range)
            .build()
            .expect("Error building the key schema partion");

        // Create the table name
        let _ = db
            .client
            .create_table()
            .table_name(TABLE_NAME)
            .billing_mode(aws_sdk_dynamodb::types::BillingMode::PayPerRequest)
            .attribute_definitions(attr_part)
            .key_schema(keyschema_part)
            .attribute_definitions(attr_sort)
            .key_schema(keyschema_sort)
            .send()
            .await;

        // TODO: handle create table result

        Self { db }
    }

    pub async fn insert(&self, project: Project) -> Result<Project, DbError> {
        let item = ProjectRepository::convert_project_to_item(&project);

        let result = self
            .db
            .client
            .put_item()
            .table_name(TABLE_NAME)
            .set_item(Some(item))
            .send()
            .await;

        match result {
            Ok(_) => Ok(project),
            Err(err) => {
                println!("An error occurred inserting project");
                println!("{:#?}", err);
                Err(DbError::UnknownError)
            }
        }
    }

    pub async fn get(&self, project_id: &str, created_by: &str) -> Result<Project, DbError> {
        let result = self
            .db
            .client
            .get_item()
            .table_name(TABLE_NAME)
            .key("created_by", AttributeValue::S(created_by.to_string()))
            .key("id", AttributeValue::S(project_id.to_string()))
            .send()
            .await;

        match result {
            Ok(output) => match output.item {
                Some(item) => match ProjectRepository::convert_item_to_project(&item) {
                    Some(project) => Ok(project),
                    None => Err(DbError::FailedConvertion(
                        "Failed converting item to project".to_string(),
                    )),
                },
                None => Err(DbError::NotFound),
            },
            Err(err) => {
                println!("{:#?}", err);
                Err(DbError::UnknownError)
            }
        }
    }

    pub async fn get_all(&self, created_by: &str) -> Result<Vec<Project>, DbError> {
        let results = self
            .db
            .client
            .query()
            .table_name(TABLE_NAME)
            .key_condition_expression("created_by = :created_by")
            .expression_attribute_values(":created_by", AttributeValue::S(created_by.to_string()))
            .send()
            .await;

        match results {
            Ok(results) => match results.items {
                Some(items) => {
                    let mut projects = Vec::new();
                    for item in items.iter() {
                        match ProjectRepository::convert_item_to_project(item) {
                            Some(project) => projects.push(project),
                            None => {
                                return Err(DbError::FailedConvertion(
                                    format!("Failed converting project item with to a project for user {created_by}").to_string(),
                                ));
                            }
                        }
                    }
                    Ok(projects)
                }
                None => Err(DbError::NotFound),
            },
            Err(err) => {
                println!("{:#?}", err);
                Err(DbError::UnknownError)
            }
        }
    }

    pub async fn update(&self, project: &Project) -> Result<Project, DbError> {
        // create a list of updates, that need to happen to the DynamoDB item
        let mut updates = vec![
            "SET name = :name",
            "SET status = :status",
            "SET total_in_seconds = :total_in_seconds",
        ];

        if let Some(_) = &project.modified_at {
            updates.push("SET modified_at = :modified_at");
        } else {
            // TODO: return modified missing
        }
        if let Some(_) = &project.modified_by {
            updates.push("SET modified_by = :modified_by");
        } else {
            // TODO: return modified missing
        }

        let update_expression = updates.join(", ");
        let item = ProjectRepository::convert_project_to_item(project);

        let result = self
            .db
            .client
            .update_item()
            .table_name(TABLE_NAME)
            .key("id", AttributeValue::S(project.id.to_string()))
            .set_update_expression(Some(update_expression))
            .set_expression_attribute_values(Some(item))
            .return_values(aws_sdk_dynamodb::types::ReturnValue::AllNew)
            .send()
            .await;

        match result {
            Ok(item) => match item.attributes {
                Some(attr) => match ProjectRepository::convert_item_to_project(&attr) {
                    Some(project) => Ok(project),
                    None => Err(DbError::FailedConvertion(
                        "Failed converting item to project".to_string(),
                    )),
                },
                None => Err(DbError::NotFound),
            },
            Err(err) => {
                println!(
                    "An error occured while updating project for id: {}",
                    project.id
                );
                println!("{:#?}", err);
                Err(DbError::UnknownError)
            }
        }
    }

    pub async fn delete(&self, project_id: &str, created_by: &str) -> Result<(), DbError> {
        let result = self
            .db
            .client
            .delete_item()
            .table_name(TABLE_NAME)
            .key("created_by", AttributeValue::S(created_by.to_string()))
            .key("id", AttributeValue::S(project_id.to_string()))
            .return_values(aws_sdk_dynamodb::types::ReturnValue::AllOld)
            .send()
            .await;

        match result {
            Ok(item) => match item.attributes {
                Some(_) => Ok(()),
                None => Err(DbError::NotFound),
            },
            Err(err) => {
                println!("{:#?}", err);
                Err(DbError::UnknownError)
            }
        }
    }

    fn convert_project_to_item(project: &Project) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();

        item.insert("id".to_string(), AttributeValue::S(project.id.to_string()));
        item.insert(
            "name".to_string(),
            AttributeValue::S(project.name.to_string()),
        );
        item.insert(
            "status".to_string(),
            AttributeValue::S(project.status.to_string()),
        );
        item.insert(
            "total_in_seconds".to_string(),
            AttributeValue::N(project.total_in_seconds.to_string()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(project.created_at.to_string()),
        );
        item.insert(
            "created_by".to_string(),
            AttributeValue::S(project.created_by.to_string()),
        );

        if let Some(modified_at) = project.modified_at.clone() {
            item.insert(
                "modified_at".to_string(),
                AttributeValue::S(modified_at.to_string()),
            );
        }

        if let Some(modified_by) = project.modified_by.clone() {
            item.insert(
                "modified_by".to_string(),
                AttributeValue::S(modified_by.to_string()),
            );
        }

        item
    }

    fn convert_item_to_project(item: &HashMap<String, AttributeValue>) -> Option<Project> {
        let id = item.get("id")?.as_s().ok()?.to_string();
        let name = item.get("name")?.as_s().ok()?.to_string();
        let status_str = item.get("status")?.as_s().ok()?.to_string();
        let status = ProjectStatus::from_str(&status_str)?;
        let total_in_seconds = item.get("total_in_seconds")?.as_n().ok()?.parse().ok()?;
        let created_at = item
            .get("created_at")?
            .as_s()
            .ok()?
            .parse::<DateTime<Utc>>()
            .ok()?;
        let created_by = item.get("created_by")?.as_s().ok()?.to_string();

        let mut modified_at: Option<DateTime<Utc>> = None;
        if let Some(modified_at_attr) = item.get("modified_at") {
            modified_at = modified_at_attr.as_s().ok()?.parse::<DateTime<Utc>>().ok();
        }
        let mut modified_by: Option<String> = None;
        if let Some(modified_by_attr) = item.get("modified_by") {
            modified_by = Some(modified_by_attr.as_s().ok()?.to_string());
        }

        Some(Project {
            id,
            name,
            status,
            total_in_seconds,
            created_at,
            created_by,
            modified_at,
            modified_by,
        })
    }
}
