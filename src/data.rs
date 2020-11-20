pub mod models;
pub mod schema;

use dotenv::dotenv;
use std::env;
use diesel::prelude::*;

embed_migrations!();

pub fn establish_connection() -> SqliteConnection {
    if cfg!(test) {
        let conn = SqliteConnection::establish(":memory:").unwrap_or_else(|_| panic!("Error creating test db!"));

        let _result = diesel_migrations::run_pending_migrations(&conn);

        conn
    } else {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be configured");

        SqliteConnection::establish(&database_url).unwrap_or_else(|_| panic!("Error creating db: {}!", database_url))
    }
}