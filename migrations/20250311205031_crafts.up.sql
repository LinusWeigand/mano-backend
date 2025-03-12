-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS crafts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    version INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

INSERT INTO crafts (name, version) VALUES ('Schreiner', 1);
INSERT INTO crafts (name, version) VALUES ('Elektriker', 1);
INSERT INTO crafts (name, version) VALUES ('Zimmerer', 1);
INSERT INTO crafts (name, version) VALUES ('Fliesenleger', 1);
