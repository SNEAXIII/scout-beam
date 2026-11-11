-- A player may give up a game: it ends unsolved (no `solved_at`) and its layout is revealed.
ALTER TYPE game_status ADD VALUE 'surrendered';
