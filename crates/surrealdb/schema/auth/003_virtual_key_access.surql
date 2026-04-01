DEFINE ACCESS OVERWRITE virtual_key_access ON DATABASE TYPE RECORD
    SIGNIN (
        SELECT * FROM virtual_api_key
        WHERE key_hash = crypto::sha256($virtual_key)
          AND enabled = true
          AND (expires_at = NONE OR expires_at > time::now())
        LIMIT 1
    )
    DURATION FOR TOKEN 1m, FOR SESSION NONE;
