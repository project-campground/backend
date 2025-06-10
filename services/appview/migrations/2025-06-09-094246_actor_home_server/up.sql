-- Add home server field to Actor
ALTER TABLE appview.actor
ADD COLUMN homeServer character varying NOT NULL;