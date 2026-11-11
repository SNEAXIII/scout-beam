-- User domain: an account that can log in.

CREATE TYPE user_role AS ENUM ('super_admin', 'admin', 'user');

CREATE TABLE "user" (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    login           VARCHAR(30) NOT NULL UNIQUE,
    role            user_role   NOT NULL DEFAULT 'user',
    last_login_date TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT user_login_length CHECK (char_length(login) BETWEEN 3 AND 30),
    CONSTRAINT user_login_charset CHECK (login ~ '^[a-zA-Z0-9]+$')
);

-- No ORM here to maintain updated_at, so the database owns it.
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_set_updated_at
    BEFORE UPDATE ON "user"
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
