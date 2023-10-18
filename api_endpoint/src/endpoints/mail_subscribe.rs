use std::sync::Arc;

use crate::{models::AppState, utils::get_error};
use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_derive::{Deserialize, Serialize};
use starknet::core::types::FieldElement;

#[derive(Serialize, Deserialize)]
pub struct MailSubscribe {
    meta_hash: String,
    tx_hash: FieldElement,
    group: String,
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
