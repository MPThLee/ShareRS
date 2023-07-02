-- Add up migration script here
-- User
CREATE TABLE IF NOT EXISTS "user" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "username" TEXT NOT NULL UNIQUE,
    "password" TEXT NOT NULL,
    "is_active" BOOLEAN NOT NULL DEFAULT true,
    "is_admin" BOOLEAN NOT NULL DEFAULT false,
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Token
CREATE TABLE IF NOT EXISTS token (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "user_id" UUID NOT NULL REFERENCES "user"("id"),
    "token" UUID NOT NULL DEFAULT gen_random_uuid(),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "expires" TIMESTAMPTZ
);

CREATE INDEX ON token("token");

-- File 
CREATE TABLE IF NOT EXISTS "file" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL UNIQUE,
    "original_name" TEXT,
    "mime" TEXT,
    "views" BIGINT DEFAULT 0,
    "max_views" BIGINT,
    "user_id" UUID NOT NULL REFERENCES "user"("id"),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ON "file"("name");

-- Url
CREATE TABLE IF NOT EXISTS "url" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL UNIQUE,
    "destination" TEXT NOT NULL,
    "views" BIGINT DEFAULT 0,
    "max_views" BIGINT,
    "user_id" UUID NOT NULL REFERENCES "user"("id"),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ON "url"("name");