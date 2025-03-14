-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS skills (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL UNIQUE,
    version SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS profile_skill (
    profile_id UUID REFERENCES profiles(id) ON DELETE CASCADE,
    skill_id UUID REFERENCES skills(id) ON DELETE CASCADE,
    PRIMARY KEY (profile_id, skill_id),
    version SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

INSERT INTO skills (name) VALUES ('Küchen');
INSERT INTO skills (name) VALUES ('Holzmöbel');
INSERT INTO skills (name) VALUES ('Elektrik');
INSERT INTO skills (name) VALUES ('Gasheizung');
INSERT INTO skills (name) VALUES ('Bad');
INSERT INTO skills (name) VALUES ('Fliesen');
INSERT INTO skills (name) VALUES ('Altbausanierung');
INSERT INTO skills (name) VALUES ('Fliesenarbeiten');
INSERT INTO skills (name) VALUES ('Trockenbau');
INSERT INTO skills (name) VALUES ('Türenmontage');
INSERT INTO skills (name) VALUES ('Fenstermontage');
INSERT INTO skills (name) VALUES ('Bodenbeläge');
INSERT INTO skills (name) VALUES ('Pflasterarbeiten');
