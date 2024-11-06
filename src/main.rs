use newslatter::configuration::config::get_configuration;
use newslatter::database::db::Database;
use newslatter::email_client::SESWorkflow;
use newslatter::routes::router::routes;
use newslatter::startup::configure_aws;
use newslatter::startup::create_aws_client;
use newslatter::startup::init_logging;
use newslatter::startup::start_server;

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    init_logging(&configuration)?;

    let shared_config = configure_aws(&configuration)?;
    let aws_client = create_aws_client(&shared_config)?;

    let ses_state = Arc::new(SESWorkflow::new(
        aws_client,
        configuration.aws.verified_email.clone(),
    ));
    let db_state = Arc::new(Database::new(configuration.database.with_db()).await?);

    let app = routes(db_state, ses_state);

    start_server(&configuration, app).await?;

    Ok(())
}
