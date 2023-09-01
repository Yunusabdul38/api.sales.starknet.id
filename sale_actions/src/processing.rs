use crate::logger::Logger;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MetadataDoc {
    pub meta_hash: String,
    pub email: String,
    pub tax_state: String,
    pub salt: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaleDoc {
    pub tx_hash: String,
    pub meta_hash: String,
    pub domain: String,
    pub price: f64,
    pub payer: String,
    pub timestamp: i64,
    pub expiry: i64,
    pub auto: bool,
    pub sponsor: Option<String>,
    pub sponsor_comm: Option<f64>,
    pub metadata: Vec<MetadataDoc>,
}

async fn process_sale(logger: &Logger, sale: &SaleDoc) {
    logger.info(format!("processing: {}", sale.domain));
}

pub async fn process_data(db: &Database, logger: &Logger) {
    let pipeline = vec![
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
                "metadata": { "$ne": [] }
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
    ];

    let sales_collection: Collection<Document> = db.collection("sales");
    let mut cursor = sales_collection.aggregate(pipeline, None).await.unwrap();
    let mut processed = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => match mongodb::bson::from_document::<SaleDoc>(document) {
                Err(e) => {
                    logger.severe(format!("Error parsing doc: {}", e));
                }
                Ok(sales_doc) => {
                    process_sale(&logger, &sales_doc).await;
                    processed.push(sales_doc.meta_hash);
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
    let processed_collection: Collection<Document> = db.collection("processed");
    match processed_collection
        .insert_many(
            processed
                .iter()
                .map(|meta_hash| doc! { "meta_hash": meta_hash })
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
