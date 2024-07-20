use super::database::{Database, DbError};
use crate::{
    models::time_track_model::{TimeTrack, TimeTrackStatus},
    User,
};
use aws_sdk_dynamodb::types::{
    AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ScalarAttributeType,
};
use chrono::{DateTime, Utc};
use std::{collections::HashMap, sync::Arc};

pub struct TimeTrackRepository {
    db: Arc<Database>,
}

static TABLE_NAME: &str = "time_trackings";

impl TimeTrackRepository {
    pub async fn new(db: Arc<Database>) -> Self {
        // Partion key: project_id
        let attr_part = AttributeDefinition::builder()
            .attribute_name("project_id")
            .attribute_type(ScalarAttributeType::S)
            .build()
            .expect(&format!(
                "Error building the attribute partion 'project_id' in the {} table",
                TABLE_NAME
            ));

        let keyschema_part = KeySchemaElement::builder()
            .attribute_name("project_id")
            .key_type(KeyType::Hash)
            .build()
            .expect(&format!(
                "Error building the key schema partion 'project_id' for table: {}",
                TABLE_NAME
            ));

        // Sort key: id
        let attr_sort = AttributeDefinition::builder()
            .attribute_name("id")
            .attribute_type(ScalarAttributeType::S)
            .build()
            .expect(&format!(
                "Error building the attribute partion 'id' in the {} table",
                TABLE_NAME
            ));

        let keyschema_sort = KeySchemaElement::builder()
            .attribute_name("id")
            .key_type(KeyType::Range)
            .build()
            .expect(&format!(
                "Error building the key schema partion 'id' for table: {}",
                TABLE_NAME
            ));

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
            .await
            .map_err(|err| println!("{:#?}", err));

        Self { db }
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
            .map_err(|err| DbError::Unknown(format!("{}: {:#?}", TABLE_NAME, err)))
    }

    pub async fn get_in_progress(&self, project_id: &str) -> Result<TimeTrack, DbError> {
        let result = self
            .db
            .client
            .get_item()
            .table_name(TABLE_NAME)
            .key("project_id", AttributeValue::S(project_id.to_string()))
            .key(
                "status",
                AttributeValue::S(TimeTrackStatus::IN_PROGRESS.to_string()),
            )
            .send()
            .await;

        match result {
            Ok(output) => match output.item {
                Some(item) => match TimeTrackRepository::convert_item_to_time_track(&item) {
                    Some(time_track) => Ok(time_track),
                    None => Err(DbError::Convertion {
                        table: TABLE_NAME.into(),
                        id: project_id.to_string(),
                    }),
                },
                None => Err(DbError::NotFound),
            },
            Err(err) => Err(DbError::Unknown(format!("{}: {:#?}", TABLE_NAME, err))),
        }
    }

    pub async fn update(&self, time_track: &TimeTrack) -> Result<(), DbError> {
        // Create a list of updates that need to happen to the DynamoDB item
        let mut updates = vec![
            "project_id = :project_id",
            "status = :status",
            "started_at = :started_at",
        ];

        if time_track.stopped_at.is_some() {
            updates.push("stopped_at = :stopped_at");
        }

        let update_expression = format!("SET {}", updates.join(", "));
        let item = TimeTrackRepository::convert_time_track_to_item(time_track);

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
            .map_err(|err| DbError::Unknown(format!("{}: {:#?}", TABLE_NAME, err)))
    }

    fn convert_time_track_to_item(tt: &TimeTrack) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();

        item.insert("id".to_string(), AttributeValue::S(tt.id.to_string()));
        item.insert(
            "project_id".to_string(),
            AttributeValue::S(tt.project_id.to_string()),
        );
        item.insert(
            "status".to_string(),
            AttributeValue::S(tt.status.to_string()),
        );
        item.insert(
            "started_at".to_string(),
            AttributeValue::S(tt.started_at.to_string()),
        );
        if let Some(stopped_at) = tt.stopped_at.clone() {
            item.insert(
                "stopped_at".to_string(),
                AttributeValue::S(stopped_at.to_string()),
            );
        }

        item
    }

    fn convert_item_to_time_track(item: &HashMap<String, AttributeValue>) -> Option<TimeTrack> {
        let id = item.get("id")?.as_s().ok()?.to_string();
        let project_id = item.get("project_id")?.as_s().ok()?.to_string();
        let status_str = item.get("status")?.as_s().ok()?;
        let status = TimeTrackStatus::from_str(&status_str)?;
        let started_at = item
            .get("started_at")?
            .as_s()
            .ok()?
            .parse::<DateTime<Utc>>()
            .ok()?;

        let mut stopped_at: Option<DateTime<Utc>> = None;
        if let Some(stopped_at_attr) = item.get("stopped_at") {
            stopped_at = stopped_at_attr.as_s().ok()?.parse::<DateTime<Utc>>().ok();
        }

        let time_track = TimeTrack {
            id,
            project_id,
            status,
            started_at,
            stopped_at,
        };

        Some(time_track)
    }
}
