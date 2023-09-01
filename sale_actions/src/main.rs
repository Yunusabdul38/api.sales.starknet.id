#[macro_use]
mod utils;
mod config;
mod logger;
mod processing;
use logger::Logger;
use mongodb::{bson::doc, options::ClientOptions, Client};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("starting v{} of sale_actions", env!("CARGO_PKG_VERSION"));
    let conf = config::load();
    let logger = Logger::new(&conf.watchtower);
    let db = Client::with_options(
        ClientOptions::parse(&conf.database.connection_string)
            .await
            .unwrap(),
    )
    .unwrap()
    .database(&conf.database.name);

    if db.run_command(doc! {"ping": 1}, None).await.is_err() {
        logger.severe("unable to connect to database");
        return;
    } else {
        logger.info("database: connected")
    }

    loop {
        processing::process_data(&db, &logger).await;
        sleep(Duration::from_secs(conf.general.check_delay)).await; // Sleep for 60 seconds before repeating
    }
}
