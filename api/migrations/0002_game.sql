-- Game domain: one attempt by a player to find a hidden layout, and the shots fired at it.

CREATE TYPE game_status AS ENUM ('in_progress', 'solved');

-- The engine's beam colours, spelled as on the wire.
CREATE TYPE beam_colour AS ENUM (
    'none', 'white', 'red', 'blue', 'yellow', 'violet', 'orange', 'green', 'black'
);

CREATE TABLE game (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    -- A game means nothing without its player.
    user_id    UUID        NOT NULL REFERENCES "user" (id) ON DELETE CASCADE,
    -- The hidden layout, as the engine serialises it; never sent while the game is in progress.
    layout     JSONB       NOT NULL,
    status     game_status NOT NULL DEFAULT 'in_progress',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    solved_at  TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT game_solved_at_matches_status CHECK ((status = 'solved') = (solved_at IS NOT NULL))
);

-- `GET /games` lists one player's games, newest first.
CREATE INDEX game_user_id_created_at_idx ON game (user_id, created_at DESC);

CREATE TRIGGER game_set_updated_at
    BEFORE UPDATE ON game
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Append-only: a shot is never updated, so it has no updated_at.
CREATE TABLE shot (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id    UUID        NOT NULL REFERENCES game (id) ON DELETE CASCADE,
    entry      SMALLINT    NOT NULL,
    exit_entry SMALLINT    NOT NULL,
    colour     beam_colour NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Re-firing an entry answers the stored shot: the index settles concurrent re-fires too.
    CONSTRAINT shot_game_id_entry_unique UNIQUE (game_id, entry),
    CONSTRAINT shot_entry_range CHECK (entry BETWEEN 1 AND 36),
    CONSTRAINT shot_exit_entry_range CHECK (exit_entry BETWEEN 1 AND 36)
);
