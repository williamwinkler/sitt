use super::{
    database::{Database, DbError},
    utils::{get_datetime_value, get_string_value},
};
use crate::models::{project_model::{Project, ProjectStatus}, user_model::User};
use aws_sdk_dynamodb::{
    error::SdkError,
    operation::create_table::CreateTableError,
    types::{AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ScalarAttributeType},
};
use chrono::{DateTime, Utc};
use humantime::{format_duration, parse_duration};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct ProjectRepository {
    db: Arc<Database>,
}

static TABLE_NAME: &str = "projects";

impl ProjectRepository {
    pub async fn build(db: Arc<Database>) -> Result<ProjectRepository, DbError> {
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

        // Create the table
        let result = db
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

        // Check if there is an error creating the table
        if let Err(SdkError::ServiceError(service_err)) = result {
            match service_err.err() {
                CreateTableError::ResourceInUseException(info) => {
                    // If the error is not, that the table already exists => throw error
                    if info
                        .message()
                        .map_or(true, |msg| !msg.contains("Table already exists"))
                    {
                        return Err(DbError::Unknown(format!("{:#?}", service_err)));
                    }
                }
                _ => return Err(DbError::Unknown(format!("{:#?}", service_err))),
            }
        }

        Ok(Self { db })
    }

    pub async fn create(&self, project: &Project) -> Result<(), DbError> {
        let item = ProjectRepository::convert_project_to_item(project);

        self.db
            .client
            .put_item()
            .table_name(TABLE_NAME)
            .set_item(Some(item))
            .send()
            .await
            .map(|_| ())
            .map_err(|err| DbError::Unknown(format!("{}: {:#?}", TABLE_NAME, err)))
    }

    pub async fn get(&self, user: &User, project_id: &str) -> Result<Project, DbError> {
        let result = self
            .db
            .client
            .get_item()
            .table_name(TABLE_NAME)
            .key("created_by", AttributeValue::S(user.id.to_string()))
            .key("id", AttributeValue::S(project_id.to_string()))
            .send()
            .await;

        match result {
            Ok(output) => match &output.item {
                Some(item) => {
                    let project = Self::convert_item_to_project(item)?;
                    Ok(project)
                },
                None => Err(DbError::NotFound),
            },
            Err(err) => Err(DbError::Unknown(format!("{}: {:#?}", TABLE_NAME, err))),
        }
    }

    pub async fn get_all(&self, user: &User) -> Result<Vec<Project>, DbError> {
        let result = self
            .db
            .client
            .query()
            .table_name(TABLE_NAME)
            .key_condition_expression("created_by = :created_by")
            .expression_attribute_values(":created_by", AttributeValue::S(user.id.to_string()))
            .send()
            .await;

        match result {
            Ok(items) => match items.items {
                Some(items) => {
                    let mut projects = Vec::new();
                    for item in items.iter() {
                        let project = Self::convert_item_to_project(item)?;
                        projects.push(project);
                    }
                    Ok(projects)
                }
                None => Err(DbError::NotFound),
            },
            Err(err) => Err(DbError::Unknown(format!("{}: {:#?}", TABLE_NAME, err))),
        }
    }

    pub async fn update(&self, user: &User, project: &mut Project) -> Result<Project, DbError> {
        // Update modified at & by
        project.modified_at = Some(Utc::now());
        project.modified_by = Some(user.id.to_string());

        let mut item = HashMap::new();

        // Create a list of updates that need to happen to the DynamoDB item
        let mut updates = vec![
            "project_name = :project_name",
            "project_status = :project_status",
            "total_duration = :total_duration",
        ];
        item.insert(
            String::from(":project_name"),
            AttributeValue::S(project.name.clone()),
        );
        item.insert(
            String::from(":project_status"),
            AttributeValue::S(project.status.to_string()),
        );
        item.insert(
            String::from(":total_duration"),
            AttributeValue::S(format_duration(project.total_duration).to_string()),
        );

        if let Some(modified_at) = project.modified_at {
            updates.push("modified_at = :modified_at");
            item.insert(
                String::from(":modified_at"),
                AttributeValue::S(modified_at.to_string()),
            );
        }
        if let Some(modified_by) = project.modified_by.clone() {
            updates.push("modified_by = :modified_by");
            item.insert(String::from(":modified_by"), AttributeValue::S(modified_by));
        }

        // Add the SET keyword only once
        let update_expression = format!("SET {}", updates.join(", "));

        let result = self
            .db
            .client
            .update_item()
            .table_name(TABLE_NAME)
            .key(
                "created_by",
                AttributeValue::S(project.created_by.to_string()),
            )
            .key("id", AttributeValue::S(project.id.to_string()))
            .update_expression(update_expression)
            .set_expression_attribute_values(Some(item))
            .return_values(aws_sdk_dynamodb::types::ReturnValue::AllNew)
            .send()
            .await;

        match result {
            Ok(item) => match &item.attributes {
                Some(attr) => {
                    let project = Self::convert_item_to_project(attr)?;
                    Ok(project)
                }
                None => Err(DbError::NotFound),
            },
            Err(err) => Err(DbError::Unknown(format!("{}: {:#?}", TABLE_NAME, err))),
        }
    }

    pub async fn delete(&self, user: &User, project_id: &str) -> Result<(), DbError> {
        let result = self
            .db
            .client
            .delete_item()
            .table_name(TABLE_NAME)
            .key("created_by", AttributeValue::S(user.id.to_string()))
            .key("id", AttributeValue::S(project_id.to_string()))
            .return_values(aws_sdk_dynamodb::types::ReturnValue::AllOld)
            .send()
            .await;

        match result {
            Ok(item) => match item.attributes {
                Some(_) => Ok(()),
                None => Err(DbError::NotFound),
            },
            Err(err) => Err(DbError::Unknown(format!("{}: {:#?}", TABLE_NAME, err))),
        }
    }

    fn convert_project_to_item(project: &Project) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();

        let key_id = String::from("id");
        let key_project_name = String::from("project_name");
        let key_status = String::from("project_status");
        let key_total_duration = String::from("total_duration");
        let key_created_at = String::from("created_at");
        let key_created_by = String::from("created_by");
        let key_modified_at = String::from("modified_at");
        let key_modified_by = String::from("modified_by");

        item.insert(key_id, AttributeValue::S(project.id.to_string()));
        item.insert(
            key_project_name,
            AttributeValue::S(project.name.to_string()),
        );
        item.insert(key_status, AttributeValue::S(project.status.to_string()));
        item.insert(
            key_total_duration,
            AttributeValue::S(format_duration(project.total_duration).to_string()),
        );
        item.insert(
            key_created_at,
            AttributeValue::S(project.created_at.to_string()),
        );
        item.insert(
            key_created_by,
            AttributeValue::S(project.created_by.to_string()),
        );

        if let Some(modified_at) = project.modified_at {
            item.insert(key_modified_at, AttributeValue::S(modified_at.to_string()));
        }

        if let Some(modified_by) = project.modified_by.clone() {
            item.insert(key_modified_by, AttributeValue::S(modified_by.to_string()));
        }

        item
    }

    fn convert_item_to_project(item: &HashMap<String, AttributeValue>) -> Result<Project, DbError> {
        let id = get_string_value(item, "id")?;
        let name = get_string_value(item, "project_name")?;
        let status = {
            let project_status_str = get_string_value(item, "project_status")?;
            project_status_str.parse::<ProjectStatus>().map_err(|_| {
                DbError::Unknown(format!(
                    "Invalid status value '{}' in item: {}",
                    project_status_str, id
                ))
            })?
        };

        let total_duration = {
            let duration_as_str = get_string_value(item, "total_duration")?;
            match parse_duration(&duration_as_str) {
                Ok(duration) => duration,
                Err(err) => {
                    return Err(DbError::Unknown(format!(
                        "Failed to parse str duration '{}' of item '{}' with err: {}",
                        duration_as_str, id, err
                    )));
                }
            }
        };
        let created_at = get_datetime_value(item, "created_at")?;
        let created_by = get_string_value(item, "created_by")?;

        let mut modified_at: Option<DateTime<Utc>> = None;
        if item.get("modified_at").is_some() {
            let modifed_at_datetime = get_datetime_value(item, "modified_at")?;
            modified_at = Some(modifed_at_datetime);
        }
        let mut modified_by: Option<String> = None;
        if item.get("modified_by").is_some() {
            let modified_by_str = get_string_value(item, "modified_by")?;
            modified_by = Some(modified_by_str)
        }

        Ok(Project {
            id,
            name,
            status,
            total_duration,
            created_at,
            created_by,
            modified_at,
            modified_by,
        })
    }
}
