-- Discord and Google sign-in: the provider ids, and the email only as a peppered hash, never in clear.
ALTER TABLE "user"
    ADD COLUMN discord_id TEXT UNIQUE,
    ADD COLUMN google_id TEXT UNIQUE,
    ADD COLUMN email_hash TEXT UNIQUE,
    ADD COLUMN email_hash_version SMALLINT,
    ADD CONSTRAINT user_email_hash_has_version CHECK ((email_hash IS NULL) = (email_hash_version IS NULL)),
    DROP CONSTRAINT user_guest_has_no_password,
    ADD CONSTRAINT user_guest_has_no_credentials
        CHECK (NOT guest OR (password_hash IS NULL AND discord_id IS NULL AND google_id IS NULL));
