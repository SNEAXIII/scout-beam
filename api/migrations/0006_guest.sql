-- Guests: accounts without credentials that may only play the daily, until they sign up.
ALTER TABLE "user" ADD COLUMN guest BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE "user" ADD CONSTRAINT user_guest_has_no_password CHECK (NOT guest OR password_hash IS NULL);
