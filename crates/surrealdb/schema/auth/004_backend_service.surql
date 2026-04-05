-- backend_service is used by the ValyMux proxy to authenticate directly to SurrealDB.
-- This access type only allows reading encrypted provider credentials via fn::backend_fetch_secret.
-- The encrypted values still require SURREAL_ENCRYPTION_KEY to decrypt (held in the Rust process).
--
-- Grant duration: 30 days. Rotate before expiry using scripts/generate_backend_grant.sh:
--   1. Run ./scripts/generate_backend_grant.sh  → new VALYMUX_DB_SERVICE_KEY
--   2. Update .env and restart the server
--   3. Revoke the old grant: ACCESS backend_service ON DATABASE REVOKE GRANT <old-id>;
DEFINE ACCESS OVERWRITE backend_service ON DATABASE TYPE BEARER FOR USER
    DURATION FOR GRANT 30d, FOR TOKEN 1h, FOR SESSION 24h;
