use super::MetadataDoc;
use crate::{config::Config, logger::Logger};
use email_address::EmailAddress;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::json;

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

async fn process_toggle_renewal(conf: &Config, logger: &Logger, sale: &ReenewalToggledDoc) {
    if !EmailAddress::is_valid(&sale.metadata[0].email) {
        logger.local(format!("email {} is not valid", &sale.metadata[0].email));
        return;
    }
    let client = reqwest::Client::new();
    // Construct the Authorization header using the api_key from the config
    let auth_header = format!("Bearer {}", &conf.email.api_key);

    // if the auto renewal was actually disabled
    if sale.allowance == "0" {
        let response = client
            .get(&format!(
                "{base_url}/subscribers/{email}",
                base_url = conf.email.base_url,
                email = &sale.metadata[0].email
            ))
            .header(header::AUTHORIZATION, &auth_header)
            .send()
            .await;
        let result = match response {
            Ok(res) => match res.json::<ApiResponse>().await {
                Ok(api_response) => api_response.data,
                Err(err) => {
                    logger.severe(format!(
                        "Error1 while trying to toggle off AR emails: {}",
                        err
                    ));
                    return;
                }
            },
            Err(err) => {
                logger.severe(format!(
                    "Error2 while trying to toggle off AR emails: {}",
                    err
                ));
                return;
            }
        };

        let new_groups = result
            .groups
            .iter()
            .map(|g| g.id.clone())
            .filter(|id| id != &conf.email.ar_group_id)
            .collect::<Vec<String>>();
        let response = client
            .put(&format!(
                "{base_url}/subscribers/{id}",
                base_url = conf.email.base_url,
                id = result.id
            ))
            .header(header::AUTHORIZATION, &auth_header)
            .json(&(json!({ "groups": new_groups })))
            .send()
            .await;
        match response {
            Ok(value) => {
                logger.info(format!("disabled ar email: {:?}", value));
            }
            Err(value) => {
                logger.severe(format!("error when disabling ar emails: {:?}", value));
            }
        }

        // default case: it is enabled, we add user to AR group
    } else {
        // Extract the groups from the MetadataDoc and format them
        let groups_params: Vec<String> = sale
            .same_tx_groups
            .iter()
            .map(|group| format!("groups[]={}", group))
            .collect();

        // Construct the URL with parameters
        let url = format!(
        "{base_url}/subscribers?email={email}&fields[name]={domain}&fields[renewer]={renewer}&{groups}",
        base_url = conf.email.base_url,
        email = &sale.metadata[0].email,
        domain = &sale.domain,
        renewer = &sale.renewer,
        groups = groups_params.join("&")
    );

        // Use reqwest to send a POST request
        match client
            .post(&url)
            .header(header::AUTHORIZATION, auth_header)
            .send()
            .await
        {
            Ok(res) => {
                if !res.status().is_success() {
                    logger.severe(format!(
                    "Received non-success status from POST request: {}. URL: {}, Response body: {}",
                    res.status(),
                    url,
                    res.text()
                        .await
                        .unwrap_or_else(|_| "Failed to retrieve response body".to_string())
                ));
                }
            }
            Err(e) => {
                logger.severe(format!("Failed to send POST request: {}", e));
            }
        }
    }
}

pub async fn process_data(conf: &Config, db: &Database, logger: &Logger) {
    let pipeline: Vec<Document> = vec![
        doc! {
            "$lookup": doc! {
                "from": "metadata",
                "localField": "meta_hash",
                "foreignField": "meta_hash",
                "as": "metadata"
            }
        },
        doc! {
            "$match": doc! {
                "metadata.meta_hash": doc! {
                  "$exists": true
                }
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "ar_processed",
                "localField": "tx_hash",
                "foreignField": "tx_hash",
                "as": "processed_doc"
            }
        },
        doc! {
            "$match": doc! {
                "processed_doc": doc! {
                    "$eq": []
                }
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "email_groups",
                "localField": "tx_hash",
                "foreignField": "tx_hash",
                "as": "same_tx_groups"
            }
        },
        doc! {
            "$addFields": doc! {
                "same_tx_groups": "$same_tx_groups.group"
            }
        },
    ];

    let collection: Collection<Document> = db.collection("auto_renew_updates");
    let mut cursor = collection.aggregate(pipeline, None).await.unwrap();
    let mut processed = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => match mongodb::bson::from_document::<ReenewalToggledDoc>(document) {
                Err(e) => {
                    logger.severe(format!("Error parsing doc in renewal: {}", e));
                }
                Ok(ar_doc) => {
                    process_toggle_renewal(&conf, &logger, &ar_doc).await;
                    processed.push(ar_doc.tx_hash);
                }
            },
            Err(e) => {
                logger.severe(format!("Error while processing: {}", e));
            }
        }
    }
    if processed.is_empty() {
        return;
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
