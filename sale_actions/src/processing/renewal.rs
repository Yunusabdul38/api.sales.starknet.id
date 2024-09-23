use super::MetadataDoc;
use crate::{config::Config, logger::Logger};
use email_address::EmailAddress;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize, Debug)]
pub struct ReenewalToggledDoc {
    pub tx_hash: String,
    pub domain: String,
    pub renewer: String,
    pub allowance: String,
    pub metadata: Vec<MetadataDoc>,
    pub same_tx_groups: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    data: Data,
}

#[derive(Deserialize, Debug)]
struct Data {
    id: String,
    groups: Vec<Group>,
}

#[derive(Deserialize, Debug)]
struct Group {
    id: String,
}

// Function to create requests for disabling auto-renewal
fn create_disable_request(data: &Data, base_url: &str, ar_group_id: &str) -> Value {
    let new_groups = data
        .groups
        .iter()
        .map(|g| g.id.clone())
        .filter(|id| id != ar_group_id)
        .collect::<Vec<String>>();

    let url = format!(
        "{base_url}/subscribers/{id}",
        base_url = base_url,
        id = data.id
    );

    json!({
        "method": "PUT",
        "path": &url,
        "body": { "groups": new_groups }
    })
}

// Function to create requests for enabling auto-renewal
fn create_enable_request(sale: &ReenewalToggledDoc, base_url: &str) -> Value {
    let groups_params: Vec<String> = sale
        .same_tx_groups
        .iter()
        .map(|group| format!("groups[]={}", group))
        .collect();

    let url = format!(
        "{base_url}/subscribers?email={email}&fields[name]={domain}&fields[renewer]={renewer}&{groups}",
        base_url = base_url,
        email = &sale.metadata[0].email,
        domain = &sale.domain,
        renewer = &sale.renewer,
        groups = groups_params.join("&")
    );

    json!({
        "method": "POST",
        "path": &url
    })
}

// Function to process batch requests
async fn process_batch_requests(conf: &Config, logger: &Logger, requests: &[Value]) {
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

// Adjusted process_data to collect renewals and process in batch
pub async fn process_data(conf: &Config, db: &Database, logger: &Logger) {
    let pipeline: Vec<Document> = vec![
        doc! {
            "$match": {
                "meta_hash": { "$exists": true },
                "tx_hash": { "$exists": true }
            }
        },
        doc! {
            "$lookup": {
                "from": "metadata",
                "let": { "meta_hash": "$meta_hash" },
                "pipeline": [
                    doc! {
                        "$match": {
                            "$expr": { "$eq": [ "$meta_hash", "$$meta_hash" ] }
                        }
                    },
                    doc! {
                        "$project": { "_id": 0, "meta_hash": 1 }
                    }
                ],
                "as": "metadata"
            }
        },
        doc! {
            "$match": {
                "metadata": { "$ne": [] }
            }
        },
        doc! {
            "$lookup": {
                "from": "ar_processed",
                "let": { "tx_hash": "$tx_hash" },
                "pipeline": [
                    doc! {
                        "$match": {
                            "$expr": { "$eq": [ "$tx_hash", "$$tx_hash" ] }
                        }
                    },
                    doc! {
                        "$project": { "_id": 0, "tx_hash": 1 }
                    }
                ],
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
                "let": { "tx_hash": "$tx_hash" },
                "pipeline": [
                    doc! {
                        "$match": {
                            "$expr": { "$eq": [ "$tx_hash", "$$tx_hash" ] }
                        }
                    },
                    doc! {
                        "$project": { "_id": 0, "group": 1 }
                    }
                ],
                "as": "same_tx_groups"
            }
        },
        doc! {
            "$project": {
                "meta_hash": 1,
                "tx_hash": 1,
                "same_tx_groups": {
                    "$map": {
                        "input": "$same_tx_groups",
                        "as": "item",
                        "in": "$$item.group"
                    }
                }
            }
        },
    ];

    let collection: Collection<Document> = db.collection("auto_renew_updates");
    let mut cursor = collection.aggregate(pipeline, None).await.unwrap();
    let mut processed = Vec::new();
    let mut batch_requests = Vec::new();
    let batch_size = conf.email.batch_size;
    let client = Client::new();

    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => match mongodb::bson::from_document::<ReenewalToggledDoc>(document) {
                Err(e) => {
                    logger.severe(format!("Error parsing doc in renewal: {}", e));
                }
                Ok(renewal_doc) => {
                    if !EmailAddress::is_valid(&renewal_doc.metadata[0].email) {
                        logger.local(format!(
                            "email {} is not valid",
                            &renewal_doc.metadata[0].email
                        ));
                        continue;
                    }

                    if renewal_doc.allowance == "0" {
                        let response = client
                            .get(&format!(
                                "{base_url}/subscribers/{email}",
                                base_url = conf.email.base_url,
                                email = &renewal_doc.metadata[0].email
                            ))
                            .header("X-MailerLite-ApiKey", &conf.email.api_key)
                            .send()
                            .await;

                        if let Ok(res) = response {
                            if let Ok(api_response) = res.json::<ApiResponse>().await {
                                batch_requests.push(create_disable_request(
                                    &api_response.data,
                                    &conf.email.base_url,
                                    &conf.email.ar_group_id,
                                ));
                            } else {
                                logger.severe(
                                    "Error parsing response while disabling AR".to_string(),
                                );
                            }
                        } else {
                            logger.severe("Error sending GET request to disable AR".to_string());
                        }
                    } else {
                        batch_requests
                            .push(create_enable_request(&renewal_doc, &conf.email.base_url));
                    }

                    if batch_requests.len() >= batch_size {
                        process_batch_requests(&conf, &logger, &batch_requests).await;
                        batch_requests.clear();
                    }
                }
            },
            Err(e) => {
                logger.severe(format!("Error while processing: {}", e));
            }
        }
    }

    if !batch_requests.is_empty() {
        process_batch_requests(&conf, &logger, &batch_requests).await;
    }

    // Blacklist the processed documents
    let processed_collection: Collection<Document> = db.collection("ar_processed");
    match processed_collection
        .insert_many(
            processed
                .iter()
                .map(|tx_hash| doc! { "tx_hash": tx_hash })
                .collect::<Vec<Document>>(),
            None,
        )
        .await
    {
        Err(e) => {
            logger.severe(format!(
                "Error inserting into 'processed' collection: {}",
                e
            ));
        }
        _ => {}
    }
}
