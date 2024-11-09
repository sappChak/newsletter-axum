use newsletter::configuration::config::get_configuration;
use newsletter::database::db::Database;
use newsletter::email_client::SESWorkflow;
use newsletter::routes::router::routes;
use newsletter::startup::configure_aws;
use newsletter::startup::create_aws_client;
use newsletter::startup::init_logging;
use newsletter::startup::start_server;

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
