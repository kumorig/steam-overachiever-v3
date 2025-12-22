-- Add unplayed_games column to run_history table
ALTER TABLE run_history ADD COLUMN IF NOT EXISTS unplayed_games INTEGER NOT NULL DEFAULT 0;
