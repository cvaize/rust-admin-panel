use r2d2::Pool;
use diesel::MysqlConnection;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use log;
use crate::Config;

pub type DbPool = Pool<ConnectionManager<MysqlConnection>>;

pub fn get_connection_pool(config: &Config) -> DbPool {
     // it from the environment within this function
    dotenv().ok();
    let database_url = config.db.mysql.url.to_owned();
    log::info!("Connecting to the database.");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    Pool::builder()
        .build(manager)
        .expect("Failed to create connection Pool.")
}