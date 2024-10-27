use sqlx::{postgres::PgPoolOptions, PgPool};

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(connection_string: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPoolOptions::new()
            .connect(connection_string)
            .await
            .expect("Failed to connect to the database!");

        Ok(Self { pool })
    }
}
