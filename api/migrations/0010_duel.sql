-- A Duel: each player hides a layout for the other, then priority passes from one to the other.

ALTER TABLE room
    ADD COLUMN host_layout     JSONB,
    ADD COLUMN opponent_layout JSONB,
    ADD COLUMN last_actor_id   UUID REFERENCES "user" (id) ON DELETE CASCADE,
    ADD COLUMN last_action_at  TIMESTAMPTZ,
    ADD CONSTRAINT room_last_actor_is_a_player CHECK (last_actor_id IN (host_id, opponent_id)),
    ADD CONSTRAINT room_last_action_is_dated
        CHECK ((last_actor_id IS NULL) = (last_action_at IS NULL));
