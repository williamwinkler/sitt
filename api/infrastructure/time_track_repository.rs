use super::{
    database::{Database, DbError},
    utils::{get_datetime_value, get_string_value},
};
use crate::models::{
    time_track_model::{TimeTrack, TimeTrackStatus},
    user_model::User,
};
use aws_sdk_dynamodb::{
    error::SdkError,
    operation::{create_table::CreateTableError, delete_item::DeleteItemError},
    types::{AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ScalarAttributeType},
};
use chrono::{DateTime, Utc};
use humantime::{format_duration, parse_duration};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[derive(Debug)]
pub struct TimeTrackRepository {
    db: Arc<Database>,
}

static TABLE_NAME: &str = "time_trackings";

impl TimeTrackRepository {
    pub async fn build(db: Arc<Database>) -> Result<Self, DbError> {
        // Partion key: project_id
        let attr_part = AttributeDefinition::builder()
            .attribute_name("project_id")
            .attribute_type(ScalarAttributeType::S)
            .build()
            .unwrap_or_else(|_| {
                panic!(
                    "Error building the attribute partion 'project_id' in the {} table",
                    TABLE_NAME
                )
            });

        let keyschema_part = KeySchemaElement::builder()
            .attribute_name("project_id")
            .key_type(KeyType::Hash)
            .build()
            .unwrap_or_else(|_| {
                panic!(
                    "Error building the key schema partion 'project_id' for table: {}",
                    TABLE_NAME
                )
            });

        // Sort key: id
        let attr_sort = AttributeDefinition::builder()
            .attribute_name("id")
            .attribute_type(ScalarAttributeType::S)
            .build()
            .unwrap_or_else(|_| {
                panic!(
                    "Error building the attribute partion 'id' in the {} table",
                    TABLE_NAME
                )
            });

        let keyschema_sort = KeySchemaElement::builder()
            .attribute_name("id")
            .key_type(KeyType::Range)
            .build()
            .unwrap_or_else(|_| {
                panic!(
                    "Error building the key schema partion 'id' for table: {}",
                    TABLE_NAME
                )
            });

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

    pub async fn create(&self, time_track: &TimeTrack) -> Result<(), DbError> {
        let item = TimeTrackRepository::convert_time_track_to_item(time_track);

        self.db
            .client
            .put_item()
            .table_name(TABLE_NAME)
            .set_item(Some(item))
            .send()
            .await
            .map(|_| ())
            .map_err(|err| DbError::Unknown(format!("{}, create(): {:#?}", TABLE_NAME, err)))
    }

    pub async fn get(
        &self,
        project_id: String,
        time_track_id: String,
    ) -> Result<TimeTrack, DbError> {
        let result = self
            .db
            .client
            .get_item()
            .table_name(TABLE_NAME)
            .key("project_id", AttributeValue::S(project_id.clone()))
            .key("id", AttributeValue::S(time_track_id))
            .send()
            .await;

        match result {
            Ok(output) => match output.item {
                Some(item) => {
                    let time_track = Self::convert_item_to_time_track(&item)?;
                    Ok(time_track)
                }
                None => Err(DbError::NotFound),
            },
            Err(err) => Err(DbError::Unknown(format!(
                "{}: get() {:#?}",
                TABLE_NAME, err
            ))),
        }
    }

    pub async fn get_in_progress(
        &self,
        user: &User,
        project_id: &str,
    ) -> Result<TimeTrack, DbError> {
        let mut expression_attribute_values = HashMap::new();
        expression_attribute_values.insert(
            ":project_id".to_string(),
            AttributeValue::S(project_id.to_string()),
        );
        expression_attribute_values.insert(
            ":time_tracking_status".to_string(),
            AttributeValue::S(TimeTrackStatus::InProgress.to_string()),
        );
        expression_attribute_values.insert(
            ":created_by".to_string(),
            AttributeValue::S(user.id.to_string()),
        );

        let result = self
            .db
            .client
            .query()
            .table_name(TABLE_NAME)
            .key_condition_expression("project_id = :project_id")
            .filter_expression(
                "time_tracking_status = :time_tracking_status AND created_by = :created_by",
            )
            .set_expression_attribute_values(Some(expression_attribute_values))
            .send()
            .await;

        match result {
            Ok(output) => {
                if let Some(items) = output.items {
                    if let Some(item) = items.first() {
                        let time_track = Self::convert_item_to_time_track(item)?;
                        Ok(time_track)
                    } else {
                        Err(DbError::NotFound)
                    }
                } else {
                    Err(DbError::NotFound)
                }
            }
            Err(err) => Err(DbError::Unknown(format!(
                "{}, get_in_progress(): {:#?}",
                TABLE_NAME, err
            ))),
        }
    }

    pub async fn get_all(&self, project_id: &str, user: &User) -> Result<Vec<TimeTrack>, DbError> {
        let mut expression_attribute_values = HashMap::new();
        expression_attribute_values.insert(
            ":project_id".to_string(),
            AttributeValue::S(project_id.to_string()),
        );
        expression_attribute_values.insert(
            ":created_by".to_string(),
            AttributeValue::S(user.id.to_string()),
        );

        let result = self
            .db
            .client
            .query()
            .table_name(TABLE_NAME)
            .key_condition_expression("project_id = :project_id")
            .filter_expression("created_by = :created_by")
            .set_expression_attribute_values(Some(expression_attribute_values))
            .send()
            .await;

        match result {
            Ok(query) => match query.items {
                Some(items) => {
                    let mut time_track_items = Vec::new();
                    for item in items.iter() {
                        let time_track = Self::convert_item_to_time_track(item)?;
                        time_track_items.push(time_track)
                    }
                    Ok(time_track_items)
                }
                None => Err(DbError::NotFound),
            },
            Err(err) => Err(DbError::Unknown(format!(
                "{}: get_all(): {:#?}",
                TABLE_NAME, err
            ))),
        }
    }

    pub async fn update(&self, time_track: &TimeTrack) -> Result<(), DbError> {
        let mut item = HashMap::new();

        // Create a list of updates that need to happen to the DynamoDB item
        let mut updates = vec![
            "time_tracking_status = :time_tracking_status",
            "started_at = :started_at",
            "total_duration = :total_duration",
        ];

        item.insert(
            String::from(":time_tracking_status"),
            AttributeValue::S(time_track.status.to_string()),
        );
        item.insert(
            String::from(":started_at"),
            AttributeValue::S(time_track.started_at.to_string()),
        );
        item.insert(
            String::from(":total_duration"),
            AttributeValue::S(format_duration(time_track.total_duration).to_string()),
        );

        if let Some(stopped_at) = time_track.stopped_at {
            updates.push("stopped_at = :stopped_at");
            item.insert(
                String::from(":stopped_at"),
                AttributeValue::S(stopped_at.to_string()),
            );
        }

        let update_expression = format!("SET {}", updates.join(", "));

        self.db
            .client
            .update_item()
            .table_name(TABLE_NAME)
            .key(
                "project_id",
                AttributeValue::S(time_track.project_id.to_string()),
            )
            .key("id", AttributeValue::S(time_track.id.to_string()))
            .update_expression(update_expression)
            .set_expression_attribute_values(Some(item))
            .send()
            .await
            .map(|_| ())
            .map_err(|err| DbError::Unknown(format!("{}, update(): {:#?}", TABLE_NAME, err)))
    }

    pub async fn delete(
        &self,
        user: &User,
        project_id: String,
        time_track_id: String,
    ) -> Result<TimeTrack, DbError> {
        let result = self
            .db
            .client
            .delete_item()
            .table_name(TABLE_NAME)
            .key("project_id", AttributeValue::S(project_id))
            .key("id", AttributeValue::S(time_track_id))
            .condition_expression("created_by = :created_by")
            .expression_attribute_values(":created_by", AttributeValue::S(user.id.clone()))
            .return_values(aws_sdk_dynamodb::types::ReturnValue::AllOld)
            .send()
            .await;

        match result {
            Ok(item) => match item.attributes {
                Some(item) => Self::convert_item_to_time_track(&item),
                None => Err(DbError::NotFound),
            },
            // If there is no time_track matching the expression dynamoDB throws this error => NotFound
            Err(SdkError::ServiceError(err)) => match err.err() {
                DeleteItemError::ConditionalCheckFailedException(_) => Err(DbError::NotFound),
                _ => Err(DbError::Unknown(format!("{:#?}", err))),
            },
            Err(err) => Err(DbError::Unknown(format!(
                "{}: delete(): {:#?}",
                TABLE_NAME, err
            ))),
        }
    }

    pub async fn delete_for_project(&self, project_id: &str) -> Result<(), DbError> {
        let mut expression_attribute_values = HashMap::new();
        expression_attribute_values.insert(
            String::from(":project_id"),
            AttributeValue::S(String::from(project_id)),
        );

        // Query all items with given project_id
        let query_result = self
            .db
            .client
            .query()
            .table_name(TABLE_NAME)
            .key_condition_expression("project_id = :project_id")
            .set_expression_attribute_values(Some(expression_attribute_values))
            .send()
            .await;

        match query_result {
            Ok(output) => {
                if let Some(items) = output.items {
                    for item in items {
                        // Delete each time track item, one by one
                        let item_project_id = item
                            .get("project_id")
                            .expect("There was no project_id on time track item");
                        let item_id = item.get("id").expect("There was no id on time track item");

                        let delete_result = self
                            .db
                            .client
                            .delete_item()
                            .table_name(TABLE_NAME)
                            .key("project_id", item_project_id.clone())
                            .key("id", item_id.clone())
                            .send()
                            .await;

                        if let Err(err) = delete_result {
                            return Err(DbError::Unknown(format!(
                                "{}, delete_for_project() delete one: {:#?}",
                                TABLE_NAME, err
                            )));
                        }
                    }
                    Ok(())
                } else {
                    Ok(())
                }
            }
            Err(err) => Err(DbError::Unknown(format!(
                "{}, delete_for_project(): {:#?}",
                TABLE_NAME, err
            ))),
        }
    }

    fn convert_time_track_to_item(tt: &TimeTrack) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();

        item.insert("id".to_string(), AttributeValue::S(tt.id.to_string()));
        item.insert(
            "project_id".to_string(),
            AttributeValue::S(tt.project_id.to_string()),
        );
        item.insert(
            "time_tracking_status".to_string(),
            AttributeValue::S(tt.status.to_string()),
        );
        if let Some(comment) = &tt.comment {
            item.insert(
                "comment".to_string(),
                AttributeValue::S(comment.to_string()),
            );
        }
        item.insert(
            "started_at".to_string(),
            AttributeValue::S(tt.started_at.to_string()),
        );
        if let Some(stopped_at) = tt.stopped_at {
            item.insert(
                "stopped_at".to_string(),
                AttributeValue::S(stopped_at.to_string()),
            );
        }
        item.insert(
            "total_duration".to_string(),
            AttributeValue::S(format_duration(tt.total_duration).to_string()),
        );
        item.insert(
            "created_by".to_string(),
            AttributeValue::S(tt.created_by.to_string()),
        );

        item
    }

    fn convert_item_to_time_track(
        item: &HashMap<String, AttributeValue>,
    ) -> Result<TimeTrack, DbError> {
        let id = get_string_value(item, "id")?;
        let project_id = get_string_value(item, "project_id")?;
        let status = {
            let status_str = get_string_value(item, "time_tracking_status")?;
            status_str.parse::<TimeTrackStatus>().map_err(|_| {
                DbError::Unknown(format!(
                    "Invalid status value '{}' for item with id {}",
                    status_str, id
                ))
            })?
        };
        let comment = get_string_value(item, "comment").ok();
        let started_at = get_datetime_value(item, "started_at")?;
        let created_by = get_string_value(item, "created_by")?;

        let mut stopped_at: Option<DateTime<Utc>> = None;
        if item.get("stopped_at").is_some() {
            let datetime = get_datetime_value(item, "stopped_at")?;
            stopped_at = Some(datetime)
        }

        let mut total_duration = Duration::new(0, 0);
        if item.get("total_duration").is_some() {
            let duration_as_str = get_string_value(item, "total_duration")?;
            total_duration = match parse_duration(&duration_as_str) {
                Ok(duration) => duration,
                Err(_) => {
                    return Err(DbError::Unknown(format!(
                        "Failed to parse str duration '{}' of item {}",
                        duration_as_str, id
                    )));
                }
            }
        } else {
            total_duration = calculate_duration_to_now(&started_at);
        }

        let time_track = TimeTrack {
            id,
            project_id,
            status,
            comment,
            started_at,
            stopped_at,
            total_duration,
            created_by,
        };

        Ok(time_track)
    }
}

fn calculate_duration_to_now(started_at: &DateTime<Utc>) -> Duration {
    let time_delta = Utc::now() - started_at;
    Duration::new(time_delta.num_seconds() as u64, 0)
}
