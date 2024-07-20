use super::database::{Database, DbError};
use crate::{models::project_model::Project, User};
use aws_sdk_dynamodb::types::{
    AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ScalarAttributeType,
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

        // Create the table
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
        // .map_err(|err| println!("{:#?}", err));

        Self { db }
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
            .key("created_by", AttributeValue::S(user.name.to_string()))
            .key("id", AttributeValue::S(project_id.to_string()))
            .send()
            .await;

        match result {
            Ok(output) => match output.item {
                Some(item) => match ProjectRepository::convert_item_to_project(&item) {
                    Some(project) => Ok(project),
                    None => Err(DbError::Convertion {
                        table: TABLE_NAME.into(),
                        id: project_id.into(),
                    }),
                },
                None => Err(DbError::NotFound),
            },
            Err(err) => {
                format!("{:#?}", err);
                Err(DbError::Unknown(format!("{}: {:#?}", TABLE_NAME, err)))
            }
        }
    }

    pub async fn get_all(&self, user: &User) -> Result<Vec<Project>, DbError> {
        let result = self
            .db
            .client
            .query()
            .table_name(TABLE_NAME)
            .key_condition_expression("created_by = :created_by")
            .expression_attribute_values(":created_by", AttributeValue::S(user.name.to_string()))
            .send()
            .await;

        match result {
            Ok(items) => match items.items {
                Some(items) => {
                    let mut projects = Vec::new();
                    for item in items.iter() {
                        match ProjectRepository::convert_item_to_project(item) {
                            Some(project) => projects.push(project),
                            None => {
                                return Err(DbError::Convertion {
                                    table: TABLE_NAME.into(),
                                    id: item
                                        .get("project_id")
                                        .expect("project_id must be present")
                                        .as_s()
                                        .expect("project_id must be a string")
                                        .into(),
                                });
                            }
                        }
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
        project.modified_by = Some(user.name.to_string());

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
            Ok(item) => match item.attributes {
                Some(attr) => match ProjectRepository::convert_item_to_project(&attr) {
                    Some(project) => Ok(project),
                    None => Err(DbError::Convertion {
                        table: TABLE_NAME.into(),
                        id: project.id.clone(),
                    }),
                },
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
            .key("created_by", AttributeValue::S(user.name.to_string()))
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
            AttributeValue::S(format_duration(project.total_duration).to_string()), // TODO: check on durations
        );
        item.insert(
            key_created_at,
            AttributeValue::S(project.created_at.to_string()),
        );
        item.insert(
            key_created_by,
            AttributeValue::S(project.created_by.to_string()),
        );

        if let Some(modified_at) = project.modified_at.clone() {
            item.insert(key_modified_at, AttributeValue::S(modified_at.to_string()));
        }

        if let Some(modified_by) = project.modified_by.clone() {
            item.insert(key_modified_by, AttributeValue::S(modified_by.to_string()));
        }

        item
    }

    fn convert_item_to_project(item: &HashMap<String, AttributeValue>) -> Option<Project> {
        let id = item.get("id")?.as_s().ok()?.to_string();
        let name = item.get("project_name")?.as_s().ok()?.to_string();
        let status = item.get("project_status")?.as_s().ok()?.parse().ok()?;
        let total_duration_str = item.get("total_duration")?.as_s().ok()?;
        let total_duration = parse_duration(total_duration_str).ok()?;
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
            total_duration,
            created_at,
            created_by,
            modified_at,
            modified_by,
        })
    }
}
