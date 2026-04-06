ALTER TABLE appview.tent_message
DROP COLUMN components;
ALTER TABLE appview.tent_message
DROP COLUMN type;

ALTER TABLE appview.tent
ALTER COLUMN type TYPE integer USING type::integer;
ALTER TABLE appview.tent
ALTER COLUMN viewType TYPE integer USING viewType::integer;