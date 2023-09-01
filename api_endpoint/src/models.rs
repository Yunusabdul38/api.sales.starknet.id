use mongodb::Database;

use crate::{config::Config, logger::Logger};

pub_struct!(;AppState {
    conf: Config,
    logger : Logger,
    db: Database,
});
