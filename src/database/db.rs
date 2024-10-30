use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(options: PgConnectOptions) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPoolOptions::new()
            .connect_with(options)
            .await
            .expect("Failed to connect to the database!");

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }
}
