CREATE TABLE dashboard_login_tokens (
    id          UUID PRIMARY KEY,
    token_hash  VARCHAR NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL
);