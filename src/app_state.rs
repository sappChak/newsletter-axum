use sqlx::{postgres::PgPoolOptions, PgPool};

pub struct AppStateInner {
    pool: PgPool,
}

impl AppStateInner {
    pub async fn new(connection_string: &str) -> Self {
        let pool = PgPoolOptions::new()
            .connect(connection_string)
            .await
            .expect("Failed to connect to the database!");

        Self { pool }
    }
}
