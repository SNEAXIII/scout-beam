-- Daily: one layout per Paris day for every player, played as an ordinary game.

CREATE TABLE daily (
    date       DATE        PRIMARY KEY,
    -- As the engine serialises it; copied into each player's game of that day.
    layout     JSONB       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Null for a game against the bot.
ALTER TABLE game ADD COLUMN daily_date DATE REFERENCES daily (date);

-- One attempt per player per day; nulls stay distinct, so games against the bot are unbounded.
CREATE UNIQUE INDEX game_user_id_daily_date_unique ON game (user_id, daily_date);
