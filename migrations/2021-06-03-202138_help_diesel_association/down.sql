-- This file should undo anything in `up.sql`
ALTER TABLE stock
RENAME COLUMN stocks_id TO stocks;
