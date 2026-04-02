-- backend_service is used by the Valymux proxy to authenticate directly to SurrealDB.
-- DURATION FOR GRANT 365d: grants are long-lived to avoid runtime re-auth (no reload hook).
-- Key rotation procedure: generate a new GRANT, update VALYMUX_DB_SERVICE_KEY in .env, restart
-- the service, then revoke the old grant. Consider shortening to 30d with automated renewal
-- for stricter production security posture.
DEFINE ACCESS OVERWRITE backend_service ON DATABASE TYPE BEARER FOR USER
    DURATION FOR GRANT 365d, FOR TOKEN 1h, FOR SESSION 24h;
