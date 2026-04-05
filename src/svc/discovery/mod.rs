mod google_genai;

use std::sync::Arc;

use tracing::{info, warn};

use crate::sys::state::AppState;

const PROVIDER_SYNC_STATUS_DELETING: &str = "deleting";

/// Runs a full model discovery sync for the given provider credential.
/// Requires a JWT token since sync operations are now user-scoped.
/// Updates sync_status on the credential throughout the process.
pub async fn run_sync(state: Arc<AppState>, token: &str, credential_id: &str) {
    // Get the credential using the JWT token (user-scoped)
    let credential = match state
        .database
        .get_provider_credential(token, credential_id)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            warn!(
                credential_id,
                "credential not found at sync start; marking as failed"
            );
            let _ = state
                .database
                .set_credential_sync_status(
                    token,
                    credential_id,
                    "failed",
                    Some("credential not found at sync start".into()),
                )
                .await;
            return;
        }
        Err(e) => {
            warn!(credential_id, error = %e, "failed to fetch credential for sync; marking as failed");
            let _ = state
                .database
                .set_credential_sync_status(token, credential_id, "failed", Some(e.to_string()))
                .await;
            return;
        }
    };

    if credential.sync_status == PROVIDER_SYNC_STATUS_DELETING {
        info!(
            credential_id,
            "skipping model sync for credential marked deleting"
        );
        return;
    }

    if let Err(e) = state
        .database
        .set_credential_sync_status(token, credential_id, "syncing", None)
        .await
    {
        warn!(credential_id, error = %e, "failed to set sync_status=syncing");
    }

    // Fetch and decrypt the provider API key via the backend_service grant.
    // This avoids holding the encrypted_api_key in the credential response.
    let api_key = match state
        .database
        .fetch_proxy_provider_api_key(&credential.id, &credential.user)
        .await
    {
        Ok(key) => key,
        Err(e) => {
            warn!(credential_id, error = %e, "failed to fetch API key for sync");
            let _ = state
                .database
                .set_credential_sync_status(token, credential_id, "failed", Some(e.to_string()))
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
                .set_credential_sync_status(token, credential_id, "failed", Some(e.to_string()))
                .await;
            return;
        }
    };

    let count = models.len();

    if !credential_is_active_for_sync(&state, token, credential_id).await {
        info!(
            credential_id,
            "aborting model sync because credential was deleted"
        );
        return;
    }

    // sync_models now takes token instead of user_id
    if let Err(e) = state
        .database
        .sync_models(token, credential_id, models)
        .await
    {
        warn!(credential_id, error = %e, "failed to persist synced models");
        let _ = state
            .database
            .set_credential_sync_status(token, credential_id, "failed", Some(e.to_string()))
            .await;
        return;
    }

    if !credential_is_active_for_sync(&state, token, credential_id).await {
        info!(
            credential_id,
            "skipping sync completion because credential was deleted"
        );
        return;
    }

    if let Err(e) = state
        .database
        .set_credential_sync_status(token, credential_id, "completed", None)
        .await
    {
        warn!(credential_id, error = %e, "failed to set sync_status=completed");
    } else {
        info!(credential_id, count, "model sync completed");
    }
}

async fn credential_is_active_for_sync(state: &AppState, token: &str, credential_id: &str) -> bool {
    match state
        .database
        .get_provider_credential(token, credential_id)
        .await
    {
        Ok(Some(credential)) => credential.sync_status != PROVIDER_SYNC_STATUS_DELETING,
        Ok(None) => false,
        Err(error) => {
            warn!(credential_id, error = %error, "failed to re-check credential during sync; marking sync as failed");
            let _ = state
                .database
                .set_credential_sync_status(token, credential_id, "failed", Some(error.to_string()))
                .await;
            false
        }
    }
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
