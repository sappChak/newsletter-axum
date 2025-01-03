use std::sync::Arc;

use newsletter::{
    configuration::config::get_configuration,
    database::db::Database,
    routes::router::router,
    ses_workflow::SESWorkflow,
    startup::{configure_sdk_config, create_aws_client, init_logging, start_server},
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    init_logging(&configuration)?;

    let sdk_config = configure_sdk_config(&configuration)?;
    let aws_client = create_aws_client(&sdk_config)?;

    let ses = Arc::new(SESWorkflow::new(
        aws_client,
        configuration.aws.verified_email.clone(),
    ));
    let db = Arc::new(Database::new(configuration.database.with_db()).await?);
    let base_url = Arc::new(configuration.application.base_url.clone());

    let state = AppState::new(db, ses);

    let app = router(state, base_url);

    start_server(&configuration, app).await?;

    Ok(())
}
