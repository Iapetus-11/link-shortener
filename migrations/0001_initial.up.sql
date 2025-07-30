CREATE TABLE platforms (
    id            UUID PRIMARY KEY,
    name          VARCHAR UNIQUE NOT NULL,
    api_key_hash  VARCHAR NOT NULL
);

CREATE TABLE links (
    slug         VARCHAR PRIMARY KEY,
    platform_id  UUID NOT NULL REFERENCES platforms (id) ON DELETE CASCADE,
    url          VARCHAR NOT NULL,
    metadata     JSONB,
    created_at   TIMESTAMPTZ NOT NULL
);

CREATE TABLE link_visits (
    link_slug    VARCHAR NOT NULL REFERENCES links (slug) ON DELETE CASCADE,
    at           TIMESTAMPTZ NOT NULL,
    headers      JSONB NOT NULL,
    ip_address   VARCHAR
);