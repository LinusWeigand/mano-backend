-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS skills (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    version INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS profile_skill (
    profile_id UUID REFERENCES profiles(id) ON DELETE CASCADE,
    skill_id UUID REFERENCES skills(id) ON DELETE CASCADE,
    PRIMARY KEY (profile_id, skill_id)
);

INSERT INTO skills (name, version) VALUES ('Küchen', 1);
INSERT INTO skills (name, version) VALUES ('Holzmöbel', 1);
INSERT INTO skills (name, version) VALUES ('Elektrik', 1);
INSERT INTO skills (name, version) VALUES ('Gasheizung', 1);
INSERT INTO skills (name, version) VALUES ('Bad', 1);
INSERT INTO skills (name, version) VALUES ('Fliesen', 1);
INSERT INTO skills (name, version) VALUES ('Altbausanierung', 1);
INSERT INTO skills (name, version) VALUES ('Fliesenarbeiten', 1);
INSERT INTO skills (name, version) VALUES ('Trockenbau', 1);
INSERT INTO skills (name, version) VALUES ('Türenmontage', 1);
INSERT INTO skills (name, version) VALUES ('Fenstermontage', 1);
INSERT INTO skills (name, version) VALUES ('Bodenbeläge', 1);
INSERT INTO skills (name, version) VALUES ('Pflasterarbeiten', 1);
