use r2d2::Pool;
use diesel::MysqlConnection;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use std::env;
use log;


pub type DbPool = Pool<ConnectionManager<MysqlConnection>>;

pub fn get_connection_pool() -> DbPool {
     // it from the environment within this function
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("no DB URL");
    log::info!("Connecting to the database.");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    Pool::builder()
        .build(manager)
        .expect("Failed to create connection Pool.")
}