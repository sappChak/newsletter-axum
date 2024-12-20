use newsletter::configuration::config::get_configuration;
use newsletter::database::db::Database;
use newsletter::routes::router::router;
use newsletter::ses_workflow::SESWorkflow;
use newsletter::startup::configure_aws;
use newsletter::startup::create_aws_client;
use newsletter::startup::init_logging;
use newsletter::startup::start_server;

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    init_logging(&configuration)?;

    let aws_config = configure_aws(&configuration)?;
    let aws_client = create_aws_client(&aws_config)?;

    let ses = Arc::new(SESWorkflow::new(
        aws_client,
        configuration.aws.verified_email.clone(),
    ));
    let db = Arc::new(Database::new(configuration.database.with_db()).await?);
    let base_url = Arc::new(configuration.application.base_url.clone());

    let app = router(db, ses, base_url);

    start_server(&configuration, app).await?;

    Ok(())
}
