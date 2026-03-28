mod google_genai;

use std::sync::Arc;

use tracing::{info, warn};

use crate::sys::state::AppState;

/// Runs a full model discovery sync for the given provider credential in the background.
/// Updates sync_status on the credential throughout the process.
pub async fn run_sync(state: Arc<AppState>, credential_id: &str) {
    let credential = match state.database.get_credential_for_sync(credential_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            warn!(credential_id, "credential not found for model sync");
            return;
        }
        Err(e) => {
            warn!(credential_id, error = %e, "failed to fetch credential for sync");
            return;
        }
    };

    if let Err(e) = state
        .database
        .set_credential_sync_status(credential_id, "syncing", None)
        .await
    {
        warn!(credential_id, error = %e, "failed to set sync_status=syncing");
    }

    let api_key = match state
        .database
        .decrypt_provider_api_key(&credential.encrypted_api_key)
    {
        Ok(key) => key,
        Err(e) => {
            warn!(credential_id, error = %e, "failed to decrypt API key for sync");
            let _ = state
                .database
                .set_credential_sync_status(credential_id, "failed", Some(e.to_string()))
                .await;
            return;
        }
    };

    let models = match discover(&credential.provider, &api_key, &state.reqwest_client).await {
        Ok(m) => m,
        Err(e) => {
            warn!(
                credential_id,
                provider = %credential.provider,
                error = %e,
                "model discovery failed"
            );
            let _ = state
                .database
                .set_credential_sync_status(credential_id, "failed", Some(e.to_string()))
                .await;
            return;
        }
    };

    let user_id = credential.user.clone();
    let count = models.len();

    if let Err(e) = state
        .database
        .sync_models(credential_id, user_id, models)
        .await
    {
        warn!(credential_id, error = %e, "failed to persist synced models");
        let _ = state
            .database
            .set_credential_sync_status(credential_id, "failed", Some(e.to_string()))
            .await;
        return;
    }

    info!(credential_id, count, "model sync completed");
}

async fn discover(
    provider: &str,
    api_key: &str,
    client: &reqwest::Client,
) -> anyhow::Result<Vec<valymux_surrealdb::ModelSyncInput>> {
    match provider {
        "google-genai" => google_genai::discover_models(api_key, client).await,
        other => anyhow::bail!("no discovery service available for provider `{other}`"),
    }
}
