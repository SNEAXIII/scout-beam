-- Password accounts: an Argon2id PHC string, NULL for every account that signs in another way.
ALTER TABLE "user" ADD COLUMN password_hash TEXT;
