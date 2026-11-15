-- A Duel: each player hides a layout for the other to find.

ALTER TABLE room
    ADD COLUMN host_layout     JSONB,
    ADD COLUMN opponent_layout JSONB;
