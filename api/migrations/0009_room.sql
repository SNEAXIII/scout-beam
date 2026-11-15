-- Rooms: two players meet through a code, then play one game each, against each other.

CREATE TYPE room_mode AS ENUM ('race', 'duel');

CREATE TYPE room_status AS ENUM ('waiting', 'composing', 'playing', 'finished');

CREATE TABLE room (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    code        TEXT        NOT NULL UNIQUE,
    mode        room_mode   NOT NULL,
    status      room_status NOT NULL DEFAULT 'waiting',
    host_id     UUID        NOT NULL REFERENCES "user" (id) ON DELETE CASCADE,
    opponent_id UUID        REFERENCES "user" (id) ON DELETE CASCADE,
    winner_id   UUID        REFERENCES "user" (id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT room_code_alphabet CHECK (code ~ '^[A-HJ-NP-Z2-9]{6}$'),
    CONSTRAINT room_opponent_is_not_host CHECK (opponent_id <> host_id),
    CONSTRAINT room_winner_is_a_player CHECK (winner_id IN (host_id, opponent_id)),
    CONSTRAINT room_winner_matches_status CHECK ((status = 'finished') = (winner_id IS NOT NULL))
);

CREATE TRIGGER room_set_updated_at
    BEFORE UPDATE ON room
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Null outside a room.
ALTER TABLE game ADD COLUMN room_id UUID REFERENCES room (id) ON DELETE CASCADE;

-- One game per player per room; nulls stay distinct, so games outside a room are unbounded.
CREATE UNIQUE INDEX game_room_id_user_id_unique ON game (room_id, user_id);

-- A room game the other player won: unsolved, its layout revealed.
ALTER TYPE game_status ADD VALUE 'lost';
