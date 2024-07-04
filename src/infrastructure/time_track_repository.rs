use std::{collections::HashMap, sync::Arc};

use aws_sdk_dynamodb::types::{
    AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ScalarAttributeType,
};

use crate::models::time_track_model::TimeTrack;

use super::{database::Database, DbErrors};

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
            .key_type(KeyType::Hash)
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
            .await;

        // TODO: handle create table result

        Self { db }
    }

    pub async fn insert(&self, time_track: TimeTrack) -> Result<TimeTrack, DbErrors> {
        let item = TimeTrackRepository::convert_time_track_to_item(&time_track);

        let result = self
            .db
            .client
            .put_item()
            .table_name(TABLE_NAME)
            .set_item(Some(item))
            .send()
            .await;

        match result {
            Ok(_) => Ok(time_track),
            Err(err) => {
                println!(
                    "An error occured inserting time_track for project: {}",
                    time_track.project_id
                );
                println!("{:#?}", err);
                Err(DbErrors::UnknownError)
            }
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
            "status".to_string(),
            AttributeValue::S(tt.status.to_string()),
        );
        item.insert(
            "started_at".to_string(),
            AttributeValue::S(tt.started_at.to_string()),
        );
        if let Some(finished_at) = tt.finished_at.clone() {
            item.insert(
                "finished_at".to_string(),
                AttributeValue::S(finished_at.to_string()),
            );
        }

        item
    }
}
