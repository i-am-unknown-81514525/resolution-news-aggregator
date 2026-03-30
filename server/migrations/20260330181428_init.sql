-- Add migration script here
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE unify(
    idx BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id TEXT UNIQUE,
    organisation TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    time timestamp NOT NULL,
    source TEXT NULL,
    score FLOAT NULL,
    link TEXT NULL,
    hash_key TEXT[],
    embedding vector(768)
);