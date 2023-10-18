use std::sync::Arc;

use crate::{
    models::AppState,
    utils::{get_error, to_hex},
};
use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_derive::{Deserialize, Serialize};
use starknet::core::types::FieldElement;

#[derive(Serialize, Deserialize)]
pub struct MailSubscribe {
    #[serde(serialize_with = "field_element_to_hex")]
    tx_hash: FieldElement,
    group: String,
}

fn field_element_to_hex<S>(fe: &FieldElement, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&to_hex(*fe))
}

#[derive(Serialize)]
pub struct Output {
    success: bool,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(query): Json<MailSubscribe>,
) -> impl IntoResponse {
    let emails_collection = state
        .db
        .collection::<mongodb::bson::Document>("email_groups");

    let bson_doc = mongodb::bson::to_bson(&query).expect("Failed to serialize to BSON");

    if let mongodb::bson::Bson::Document(document) = bson_doc {
        match emails_collection.insert_one(document, None).await {
            Ok(_) => (),
            Err(err) => return get_error(format!("Failed to insert document: {}", err)),
        }
    } else {
        return get_error("Failed to create BSON document".to_string());
    }

    (StatusCode::OK, Json(Output { success: true })).into_response()
}
