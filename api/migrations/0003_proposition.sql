-- Propositions: layouts a player submitted as the answer to one of their games.

-- Append-only: a proposition is never updated, so it has no updated_at.
CREATE TABLE proposition (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    -- A proposition means nothing without its game.
    game_id    UUID        NOT NULL REFERENCES game (id) ON DELETE CASCADE,
    -- As the engine serialises it; validated before it is stored.
    layout     JSONB       NOT NULL,
    correct    BOOLEAN     NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The cascade from `game` looks propositions up by game.
CREATE INDEX proposition_game_id_idx ON proposition (game_id);

-- A correct proposition solves its game, so a game has at most one.
CREATE UNIQUE INDEX proposition_one_correct_per_game ON proposition (game_id) WHERE correct;
