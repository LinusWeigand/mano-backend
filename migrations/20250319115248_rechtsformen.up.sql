-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS rechtsformen (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL UNIQUE,
    explain_name VARCHAR(100) NOT NULL UNIQUE,
    version SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

INSERT INTO rechtsformen (name, explain_name) VALUES ('Kleingewerbe', 'Kleingewerbe');
INSERT INTO rechtsformen (name, explain_name) VALUES ('Einzelunternehmen', 'Einzelunternehmen');
INSERT INTO rechtsformen (name, explain_name) VALUES ('GbR', 'Gesellschaft bürgerlichen Rechts (GbR)');
INSERT INTO rechtsformen (name, explain_name) VALUES ('AG', 'Aktiengesellschaft (AG)');
INSERT INTO rechtsformen (name, explain_name) VALUES ('GmbH', 'Gesellschaft mit beschränkter Haftung (GmbH)');
INSERT INTO rechtsformen (name, explain_name) VALUES ('UG', 'Unternehmergesellschaft (UG)');
INSERT INTO rechtsformen (name, explain_name) VALUES ('OHG', 'Offene Handelsgesellschaft (OHG)');
INSERT INTO rechtsformen (name, explain_name) VALUES ('KG', 'Kommanditgesellschaft (KG)');
INSERT INTO rechtsformen (name, explain_name) VALUES ('PartG', 'Partnerschaftsgesellschaft (PartG)');
INSERT INTO rechtsformen (name, explain_name) VALUES ('EWIV', 'Europäische Wirtschaftliche Interessenvereinigung (EWIV)');
INSERT INTO rechtsformen (name, explain_name) VALUES ('eG', 'Eingetragene Genossenschaft (eG)');
INSERT INTO rechtsformen (name, explain_name) VALUES ('Freiberufler', 'Freiberufler');
INSERT INTO rechtsformen (name, explain_name) VALUES ('', 'Andere');
