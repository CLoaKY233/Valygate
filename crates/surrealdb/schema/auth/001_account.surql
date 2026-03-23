DEFINE ACCESS IF NOT EXISTS account ON DATABASE TYPE RECORD
    SIGNUP (
        CREATE user CONTENT {
            name: $name,
            email: $email,
            password: crypto::argon2::generate($password),
            enabled: true
        }
    )
    SIGNIN (
        SELECT * FROM user
        WHERE email = $email
            AND crypto::argon2::compare(password, $password)
            AND enabled = true
    )
    DURATION FOR TOKEN 15m, FOR SESSION 12h;
