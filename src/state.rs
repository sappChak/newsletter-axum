use std::sync::Arc;

use axum::extract::FromRef;

use crate::{database::db::Database, ses_workflow::SESWorkflow};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub workflow: Arc<SESWorkflow>,
}

impl AppState {
    pub fn new(db: Arc<Database>, workflow: Arc<SESWorkflow>) -> Self {
        Self { db, workflow }
    }
}

impl FromRef<AppState> for Arc<Database> {
    fn from_ref(app_state: &AppState) -> Arc<Database> {
        app_state.db.clone()
    }
}

impl FromRef<AppState> for Arc<SESWorkflow> {
    fn from_ref(app_state: &AppState) -> Arc<SESWorkflow> {
        app_state.workflow.clone()
    }
}
