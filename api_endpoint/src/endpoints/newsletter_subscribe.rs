use std::sync::Arc;

use crate::{models::AppState, utils::get_error};
use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AddNewsletterQuery {
    email: String,
    address: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AddNewsletterRecord {
    email: String,
    address: Option<String>,
    source: String,
}

#[derive(Serialize)]
pub struct Output {
    success: bool,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(query): Json<AddNewsletterQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<mongodb::bson::Document>("newsletter");

    // Check if email already exists
    let filter = mongodb::bson::doc! { "email": &query.email };
    let result = collection
        .find_one(filter, None)
        .await
        .expect("Failed to execute find_one");

    if let Some(_) = result {
        return get_error("Email already exists".to_string());
    }

    let bson_doc = mongodb::bson::to_bson(&AddNewsletterRecord {
        email: query.email,
        address: query.address,
        source: "newsletter_subscription".to_string(),
    })
    .expect("Failed to serialize to BSON");

    if let mongodb::bson::Bson::Document(document) = bson_doc {
        match collection.insert_one(document, None).await {
            Ok(_) => (),
            Err(err) => return get_error(format!("Failed to insert document: {}", err)),
        }
    } else {
        return get_error("Failed to create BSON document".to_string());
    }

    (StatusCode::OK, Json(Output { success: true })).into_response()
}
