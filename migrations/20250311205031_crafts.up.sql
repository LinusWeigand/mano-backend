-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS crafts (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL UNIQUE,
    version SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

INSERT INTO crafts (name) VALUES ('Schreiner');
INSERT INTO crafts (name) VALUES ('Elektriker');
INSERT INTO crafts (name) VALUES ('Zimmerer');
INSERT INTO crafts (name) VALUES ('Fliesenleger');
INSERT INTO crafts (name) VALUES ('Maler');
