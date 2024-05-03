use super::MetadataDoc;
use crate::{config::Config, logger::Logger};
use chrono::NaiveDateTime;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize, Debug)]
pub struct SaleDoc {
    pub tx_hash: String,
    pub domain: String,
    pub price: f64,
    pub payer: String,
    pub timestamp: i64,
    pub expiry: i64,
    pub metadata: Vec<MetadataDoc>,
    pub same_tx_groups: Vec<String>, // The new field
}

// Adjusted process_sale to create a request object instead of directly sending
fn create_sale_request(sale: &SaleDoc, base_url: &str) -> Value {
    let groups_params: Vec<String> = sale
        .same_tx_groups
        .iter()
        .map(|group| format!("groups[]={}", group))
        .collect();

    let url = format!(
        "{base_url}/subscribers?email={email}&fields[name]={domain}&fields[expiry]={expiry}&{groups}",
        base_url = base_url,
        email = urlencoding::encode(&sale.metadata[0].email),
        domain = urlencoding::encode(&sale.domain),
        expiry = match NaiveDateTime::from_timestamp_opt(sale.expiry, 0) {
            Some(time) => urlencoding::encode(&time.format("%Y-%m-%d %H:%M:%S").to_string()).to_string(),
            _ => "none".to_string(),
        },
        groups = groups_params.join("&")
    );

    json!({
        "method": "POST",
        "path": &url,
    })
}

// process batch requests
async fn process_batch(conf: &Config, logger: &Logger, sales: &[SaleDoc]) {
    let requests: Vec<Value> = sales
        .iter()
        .map(|sale| create_sale_request(sale, &conf.email.base_url))
        .collect();

    let batch_request = json!({
        "requests": requests
    });

    let client = Client::new();
    match client
        .post("https://api.mailerlite.com/api/v2/batch")
        .header("X-MailerLite-ApiKey", &conf.email.api_key)
        .header(header::CONTENT_TYPE, "application/json")
        .json(&batch_request)
        .send()
        .await
    {
        Ok(res) => {
            if !res.status().is_success() {
                logger.severe(format!(
                    "Received non-success status from batch request: {}. Response body: {}",
                    res.status(),
                    res.text()
                        .await
                        .unwrap_or_else(|_| "Failed to retrieve response body".to_string())
                ));
            }
        }
        Err(e) => {
            logger.severe(format!("Failed to send batch request: {}", e));
        }
    }
}

// collect sales and process in batch
pub async fn process_data(conf: &Config, db: &Database, logger: &Logger) {
    let pipeline: Vec<Document> = vec![
        doc! {
            "$lookup": {
                "from": "metadata",
                "localField": "meta_hash",
                "foreignField": "meta_hash",
                "as": "metadata"
            }
        },
        doc! {
            "$match": {
                "metadata.meta_hash": doc! {
                  "$exists": true
                }
            }
        },
        doc! {
            "$lookup": {
                "from": "processed",
                "localField": "meta_hash",
                "foreignField": "meta_hash",
                "as": "processed_doc"
            }
        },
        doc! {
            "$match": {
                "processed_doc": { "$eq": [] }
            }
        },
        doc! {
            "$lookup": {
                "from": "email_groups",
                "localField": "tx_hash",
                "foreignField": "tx_hash",
                "as": "same_tx_groups"
            }
        },
        // Optional: If you only want the 'group' field from the same_tx_groups
        doc! {
            "$addFields": {
                "same_tx_groups": "$same_tx_groups.group"
            }
        },
    ];
    let sales_collection: Collection<Document> = db.collection("sales");
    let mut cursor = sales_collection.aggregate(pipeline, None).await.unwrap();
    let mut batch = Vec::new();
    let batch_size = conf.email.batch_size;
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => match mongodb::bson::from_document::<SaleDoc>(document) {
                Err(e) => {
                    logger.severe(format!("Error parsing doc in purchase: {}", e));
                }
                Ok(sales_doc) => {
                    batch.push(sales_doc);
                    if batch.len() >= batch_size {
                        process_batch(&conf, &logger, &batch).await;
                        batch.clear();
                    }
                }
            },
            Err(e) => {
                logger.severe(format!("Error while processing: {}", e));
            }
        }
    }

    // Process any remaining sales not reaching batch size
    if !batch.is_empty() {
        process_batch(&conf, &logger, &batch).await;
    }
}
