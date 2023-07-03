-- Add up migration script here
-- User
CREATE TABLE IF NOT EXISTS users (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "username" TEXT NOT NULL UNIQUE,
    "password" TEXT NOT NULL,
    "is_active" BOOLEAN NOT NULL DEFAULT true,
    "is_admin" BOOLEAN NOT NULL DEFAULT false,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Token
CREATE TABLE IF NOT EXISTS token (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "user_id" UUID NOT NULL REFERENCES users("id"),
    "expires" TIMESTAMPTZ,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ON token("id");

-- File 
CREATE TABLE IF NOT EXISTS files (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL UNIQUE,
    "original_name" TEXT,
    "mime" TEXT,
    "views" BIGINT NOT NULL DEFAULT 0,
    "max_views" BIGINT,
    "is_processing" BOOLEAN NOT NULL DEFAULT true,
    "user_id" UUID NOT NULL REFERENCES users("id"),
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ON files("name");

-- Url
CREATE TABLE IF NOT EXISTS "url" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL UNIQUE,
    "destination" TEXT NOT NULL,
    "views" BIGINT NOT NULL DEFAULT 0,
    "max_views" BIGINT,
    "user_id" UUID NOT NULL REFERENCES users("id"),
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ON "url"("name");