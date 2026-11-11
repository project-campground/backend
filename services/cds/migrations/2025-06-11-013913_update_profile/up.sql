-- Add location, tagline & createdAt fields to Profile
ALTER TABLE appview.profile
ADD COLUMN IF NOT EXISTS location character varying;
ALTER TABLE appview.profile
ADD COLUMN IF NOT EXISTS tagline character varying;
ALTER TABLE appview.profile
ADD COLUMN IF NOT EXISTS createdAt character varying;
