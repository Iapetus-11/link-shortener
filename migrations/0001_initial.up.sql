CREATE TABLE platforms (
    id            UUID PRIMARY KEY,
    name          VARCHAR NOT NULL,
    api_key_hash  VARCHAR NOT NULL
);

CREATE TABLE links (
    id           UUID PRIMARY KEY,
    platform_id  UUID NOT NULL REFERENCES platforms (id) ON DELETE CASCADE,
    slug         VARCHAR NOT NULL,
    url          VARCHAR NOT NULL,
    metadata     JSONB,
    created_at   TIMESTAMPTZ NOT NULL
);

CREATE TABLE link_visits (
    link_id      UUID NOT NULL REFERENCES links (id) ON DELETE CASCADE,
    at           TIMESTAMPTZ NOT NULL,
    headers      JSONB,
    ip_address   VARCHAR
);