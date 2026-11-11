ALTER TABLE appview.bonfire
ADD COLUMN IF NOT EXISTS home boolean NOT NULL
DEFAULT false;

WITH homes AS (
    -- Get top of them, since that is how old back-end did it
    SELECT array_agg(bonfiresbypriority[1])
    FROM (
        -- List of bonfire IDs by priority
        SELECT array_agg(id ORDER BY priority) as bonfiresbypriority
        FROM appview.bonfire
        GROUP BY campsiteid
    )
)
UPDATE appview.bonfire
-- Add homes; should be unique per campsite
SET home = true
WHERE id = ANY((SELECT * FROM homes)::Text[]);