use crate::routes::{health_check, subscribe};
use axum::routing::{get, post};
use axum::Router;
use sqlx::PgConnection;
use tokio::net::TcpListener;


// Separated for testing purposes

pub async fn run(listener: TcpListener, pg_connection: PgConnection) {
    
    
}
