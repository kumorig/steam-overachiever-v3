-- Run history table (tracks each sync/update run)
CREATE TABLE IF NOT EXISTS run_history (
    id SERIAL PRIMARY KEY,
    steam_id BIGINT REFERENCES users(steam_id) ON DELETE CASCADE,
    run_at TIMESTAMPTZ DEFAULT NOW(),
    total_games INTEGER NOT NULL
);

-- Achievement history table (tracks achievement progress over time)
CREATE TABLE IF NOT EXISTS achievement_history (
    id SERIAL PRIMARY KEY,
    steam_id BIGINT REFERENCES users(steam_id) ON DELETE CASCADE,
    recorded_at TIMESTAMPTZ DEFAULT NOW(),
    total_achievements INTEGER NOT NULL,
    unlocked_achievements INTEGER NOT NULL,
    games_with_achievements INTEGER NOT NULL,
    avg_completion_percent DOUBLE PRECISION NOT NULL
);

-- Indexes for history queries
CREATE INDEX IF NOT EXISTS idx_run_history_steam_id ON run_history(steam_id);
CREATE INDEX IF NOT EXISTS idx_achievement_history_steam_id ON achievement_history(steam_id);
