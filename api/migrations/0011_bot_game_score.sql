-- When a game was surrendered, so a Bot game's score can be timed like a solved one.
ALTER TABLE game ADD COLUMN surrendered_at TIMESTAMPTZ;
UPDATE game SET surrendered_at = updated_at WHERE status = 'surrendered';
ALTER TABLE game ADD CONSTRAINT game_surrendered_at_matches_status
    CHECK ((status = 'surrendered') = (surrendered_at IS NOT NULL));

CREATE FUNCTION points(score int8) RETURNS int8
    LANGUAGE sql IMMUTABLE
    RETURN GREATEST(0, round(1000 - 200 * ln(1 + score / 60.0)))::int8;
