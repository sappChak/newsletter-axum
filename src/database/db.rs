use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(options: PgConnectOptions) -> Result<Self, anyhow::Error> {
        let pool = PgPoolOptions::new().connect_lazy_with(options);
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }
}
