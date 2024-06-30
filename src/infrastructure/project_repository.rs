use crate::models::project_model::Project;

use super::database::Database;
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
        // Sort Key: id

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

        // TODO: handle create result -> it fails since the table already exists

        Self { db }
    }

    pub async fn insert(&self, project: Project) -> Result<Project, String> {
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
                Err("An error occurred inserting project".to_string())
            }
        }
    }

    pub async fn get_all(&self, created_by: String) -> Result<Vec<Project>, String> {
        let results = self
            .db
            .client
            .query()
            .table_name(TABLE_NAME)
            .key_condition_expression("created_by = :created_by")
            .expression_attribute_values(":created_by", AttributeValue::S(created_by))
            .send()
            .await
            .expect("Failed to retrieve projects");

        if let Some(items) = results.items {
            let projects: Vec<Project> = items.iter().map(|v: &HashMap<String, AttributeValue>| v.into()).collect();
            Ok(projects)
        } else {
            Err(format!("An error occurred querying projects"))
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
}

//     pub async fn get_all(&self, created_by: &str) -> Result<Vec<EProject>, String> {
//         let query = self
//             .db
//             .client
//             .query()
//             .table_name(TABLE_NAME)
//             .key_condition_expression("created_by = :created_by")
//             .expression_attribute_values(":created_by", AttributeValue::S(created_by.to_string()))
//             .send()
//             .await;

//         match query {
//             Ok(result) => {
//                 if let Some(items) = result.items {
//                     let projects: Vec<EProject> = items
//                         .iter()
//                         .filter_map(|item| {
//                             let status = item.get("status")?.as_s().ok()?;
//                             println!("{}", status);
//                             match status.as_str() {
//                                 "ACTIVE" => {
//                                     let project =
//                                         ProjectRepository::convert_to_project(item, Active)?;
//                                     Some(EProject::from(project))
//                                 }
//                                 "INACTIVE" => {
//                                     let project =
//                                         ProjectRepository::convert_to_project(item, Inactive)
//                                             .expect("Failed to convert to project");

//                                     println!("{}", project.name);
//                                     Some(EProject::from(project))
//                                 }
//                                 _ => None,
//                             }
//                         })
//                         .collect();
//                     Ok(projects)
//                 } else {
//                     Err("No projects found".to_string())
//                 }
//             }
//             Err(err) => {
//                 println!(
//                     "Error occurred while querying projects for '{}': {:#?}",
//                     created_by, err
//                 );
//                 Err("Error occurred while querying projects".to_string())
//             }
//         }
//     }

//     fn get_status_as_str(&self, project: EProject) -> String {
//         match project {
//             EProject::Active(_) => String::from("ACTIVE"),
//             EProject::Inactive(_) => String::from("INACTIVE"),
//         }
//     }

//     fn convert_to_project<Status>(
//         item: &HashMap<String, AttributeValue>,
//         status: Status,
//     ) -> Option<Project<Status>> {
//         Some(Project {
//             id: item
//                 .get("id")
//                 .expect("id missing")
//                 .as_s()
//                 .expect("id not a string")
//                 .to_string(),
//             name: item
//                 .get("name")
//                 .expect("name missing")
//                 .as_s()
//                 .expect("name not a string")
//                 .to_string(),
//             status,
//             total_in_seconds: item
//                 .get("total_in_seconds")
//                 .expect("total_in_seconds missing")
//                 .as_n()
//                 .expect("total_in_seconds not a number")
//                 .parse()
//                 .expect("failed to parse total_in_seconds"),
//             created_at: item
//                 .get("created_at")
//                 .expect("created_at missing")
//                 .as_s()
//                 .expect("created_at not a string")
//                 .parse()
//                 .expect("failed to parse created_at"),
//             created_by: item
//                 .get("created_by")
//                 .expect("created_by missing")
//                 .as_s()
//                 .expect("created_by not a string")
//                 .to_string(),
//             modified_by: item
//                 .get("modified_by")
//                 .map(|attr| attr.as_s().expect("modified_by not a string").to_string()),
//             modified_at: item.get("modified_at").map(|attr| {
//                 attr.as_s()
//                     .expect("modified_at not a string")
//                     .parse()
//                     .expect("failed to parse modified_at")
//             }),
//         })
//     }
// }

// let status = self.get_status_as_str(EProject::from(project));
// let total_in_seconds = project.total_in_seconds.clone().to_string();

// let result = self
//     .db
//     .client
//     .put_item()
//     .table_name(TABLE_NAME)
//     .item("id", S(project.id.to_string()))
//     .item("name", S(project.name.to_string()))
//     .item("status", S(status))
//     .item("total_in_seconds", N(total_in_seconds))
//     .item("created_at", S(project.created_at.to_string()))
//     .item("created_by", S(project.created_by.to_string()))
//     .send()
//     .await;
