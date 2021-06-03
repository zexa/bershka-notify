-- Your SQL goes here
ALTER TABLE stocks
ALTER COLUMN product_id TYPE INTEGER;

ALTER TABLE stocks
ALTER COLUMN product_id SET NOT NULL;

