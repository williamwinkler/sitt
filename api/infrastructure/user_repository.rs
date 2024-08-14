use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::models::user_model::{User, UserRole};
use aws_sdk_dynamodb::{
    error::SdkError,
    operation::create_table::CreateTableError,
    types::{AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ScalarAttributeType},
};
use tokio::time::sleep;

use super::{
    database::{Database, DbError},
    utils::{get_datetime_value, get_string_value},
};

#[derive(Debug)]
pub struct UserRepository {
    db: Arc<Database>,
}

static TABLE_NAME: &str = "users";

impl UserRepository {
    pub async fn build(db: Arc<Database>) -> Result<UserRepository, DbError> {
        // Partition key: api_key
        let attr_part = AttributeDefinition::builder()
            .attribute_name("api_key")
            .attribute_type(ScalarAttributeType::S)
            .build()
            .map_err(|e| {
                DbError::Unknown(format!(
                    "Error building the attribute partition 'api_key' in the 'users' table: {:?}",
                    e
                ))
            })?;

        let keyschema_part = KeySchemaElement::builder()
            .attribute_name("api_key")
            .key_type(KeyType::Hash)
            .build()
            .map_err(|e| {
                DbError::Unknown(format!(
                    "Error building the key schema partition 'api_key': {:?}",
                    e
                ))
            })?;

        // Sort key: id
        let attr_sort = AttributeDefinition::builder()
            .attribute_name("id")
            .attribute_type(ScalarAttributeType::S)
            .build()
            .map_err(|e| {
                DbError::Unknown(format!(
                    "Error building the attribute partition 'id' in the 'users' table: {:?}",
                    e
                ))
            })?;

        let keyschema_sort = KeySchemaElement::builder()
            .attribute_name("id")
            .key_type(KeyType::Range)
            .build()
            .map_err(|e| {
                DbError::Unknown(format!(
                    "Error building the key schema partition 'id': {:?}",
                    e
                ))
            })?;

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
        if let Err(SdkError::ServiceError(service_err)) = &result {
            match service_err.err() {
                CreateTableError::ResourceInUseException(info) => {
                    // If the error is not, that the table already exists => throw error
                    if info
                        .message()
                        .map_or(true, |msg| !msg.contains("Table already exists"))
                    {
                        return Err(DbError::Unknown(format!(
                            "CreateTableError::ResourceInUseException: {:#?}",
                            info
                        )));
                    }
                }
                _ => return Err(DbError::Unknown(format!("{:#?}", service_err))),
            }
        }

        let user_repository = Self { db };

        // If the table was just created, add a default admin user
        if result.is_ok() {
            let default_admin_user = User::new("DEFAULT ADMIN", &UserRole::Admin, "SYSTEM");
            let one_sec = Duration::new(1, 0);
            let max_retries = 30;
            let mut attempt = 0;

            // Loop because it can take some time for the DynamoDB table to get created
            loop {
                if attempt >= max_retries {
                    eprintln!("Failed to create DEFAULT ADMIN user after {} attempts. Exiting.", max_retries);
                    std::process::exit(1);
                }
                println!("Trying to create default admin user (attempt {})...", attempt + 1);
                let result = user_repository.create(&default_admin_user).await;
                if result.is_ok() {
                    break;
                } else {
                    sleep(one_sec).await;
                    attempt += 1;
                }
            }
            println!("Created DEFAULT ADMIN user. Make sure to create your own ADMIN user and delete this one.");
        }

        Ok(user_repository)
    }

    pub async fn create(&self, user: &User) -> Result<(), DbError> {
        let item = Self::convert_user_to_item(user);

        let result = self
            .db
            .client
            .put_item()
            .table_name(TABLE_NAME)
            .set_item(Some(item))
            .send()
            .await;

        if let Err(e) = result {
            return Err(DbError::Unknown(format!("Error putting item: {:?}", e)));
        }

        Ok(())
    }

    pub async fn get_by_api_key(&self, api_key: &str) -> Result<User, DbError> {
        let result = self
            .db
            .client
            .query()
            .table_name(TABLE_NAME)
            .key_condition_expression("api_key = :api_key")
            .expression_attribute_values(":api_key", AttributeValue::S(api_key.to_string()))
            .send()
            .await;

        match result {
            Ok(output) => {
                if let Some(items) = output.items {
                    if !items.is_empty() {
                        let user = Self::convert_item_to_user(&items[0])?;
                        Ok(user)
                    } else {
                        Err(DbError::NotFound)
                    }
                } else {
                    Err(DbError::NotFound)
                }
            }
            Err(err) => Err(DbError::Unknown(format!(
                "{}: get_by_api_key {:#?}",
                TABLE_NAME, err
            ))),
        }
    }

    pub async fn get_by_id(&self, id: &str) -> Result<User, DbError> {
        let result = self
            .db
            .client
            .scan()
            .table_name(TABLE_NAME)
            .filter_expression("#id = :id")
            .expression_attribute_names("#id", "id")
            .expression_attribute_values(":id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|err| DbError::Unknown(format!("Error scanning table: {:?}", err)))?;

        if let Some(items) = result.items {
            if !items.is_empty() {
                let user = Self::convert_item_to_user(&items[0])?;
                Ok(user)
            } else {
                Err(DbError::NotFound)
            }
        } else {
            Err(DbError::NotFound)
        }
    }

    pub async fn get_all(&self) -> Result<Vec<User>, DbError> {
        let result = self
            .db
            .client
            .scan()
            .table_name(TABLE_NAME)
            .send()
            .await
            .map_err(|err| {
                DbError::Unknown(format!(
                    "Something went wrong when extracting all users: {:#?}",
                    err
                ))
            })?;

        if let Some(items) = result.items {
            if !items.is_empty() {
                let mut users: Vec<User> = Vec::new();
                for item in items.iter() {
                    let user = Self::convert_item_to_user(item)?;
                    users.push(user);
                }
                return Ok(users);
            } else {
                return Err(DbError::NotFound);
            }
        } else {
            return Err(DbError::NotFound);
        }
    }

    pub async fn delete(&self, user: &User) -> Result<(), DbError> {
        let api_key = match &user.api_key {
            Some(key) => key.clone(),
            None => return Err(DbError::Unknown(format!("User api_key is missing"))),
        };

        let result = self
            .db
            .client
            .delete_item()
            .table_name(TABLE_NAME)
            .key("api_key", AttributeValue::S(api_key))
            .key("id", AttributeValue::S(user.id.to_string()))
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

    fn convert_user_to_item(user: &User) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();

        item.insert("id".to_string(), AttributeValue::S(user.id.clone()));
        item.insert(
            "user_name".to_string(),
            AttributeValue::S(user.name.clone()),
        );
        item.insert(
            "user_role".to_string(),
            AttributeValue::S(user.role.to_string()),
        );
        item.insert(
            "api_key".to_string(),
            AttributeValue::S(user.api_key.clone().unwrap()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(user.created_at.to_rfc3339()),
        );
        item.insert(
            "created_by".to_string(),
            AttributeValue::S(user.created_by.clone()),
        );

        item
    }

    fn convert_item_to_user(item: &HashMap<String, AttributeValue>) -> Result<User, DbError> {
        let id = get_string_value(item, "id")?;
        let name = get_string_value(item, "user_name")?;
        let role = {
            let role_str = get_string_value(item, "user_role")?;
            role_str.parse::<UserRole>().map_err(|_| {
                DbError::Unknown(format!("Invalid role value '{}' in item: {}", role_str, id))
            })?
        };
        let api_key = {
            let api_key = get_string_value(item, "api_key")?;
            Some(api_key)
        };
        let created_at = get_datetime_value(item, "created_at")?;
        let created_by = get_string_value(item, "created_by")?;

        Ok(User {
            id,
            name,
            role,
            api_key,
            created_at,
            created_by,
        })
    }
}
