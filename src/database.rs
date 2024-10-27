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

        let _ = sqlx::query!(
            "CREATE TABLE IF NOT EXISTS subscriptions(
             id uuid NOT NULL,
             PRIMARY KEY (id),
             email TEXT NOT NULL UNIQUE,
             name TEXT NOT NULL,
             subscribed_at timestamptz NOT NULL);"
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}
