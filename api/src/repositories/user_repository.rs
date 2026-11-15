//! Every SQL statement of the user domain, and nothing else.
//!
//! `query_as!` checks each one against a live schema at compile time.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user_model::User;
use crate::shared::role::Role;

pub async fn find_by_id(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"SELECT id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at
           FROM "user"
           WHERE id = $1"#,
        user_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_by_login(pool: &PgPool, login: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"SELECT id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at
           FROM "user"
           WHERE login = $1"#,
        login
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert(pool: &PgPool, login: &str, role: Role) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"INSERT INTO "user" (login, role)
           VALUES ($1, $2)
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        login,
        role as _
    )
    .fetch_one(pool)
    .await
}

/// A password account: the role is the column default, `user`.
pub async fn insert_with_password(
    pool: &PgPool,
    login: &str,
    password_hash: &str,
) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"INSERT INTO "user" (login, password_hash)
           VALUES ($1, $2)
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        login,
        password_hash
    )
    .fetch_one(pool)
    .await
}

/// Insert, unless the login was taken meanwhile: `None` means "someone else won the race".
/// `DO NOTHING` rather than `DO UPDATE`, so the existing row and its `updated_at` stay untouched.
pub async fn insert_if_absent(
    pool: &PgPool,
    login: &str,
    role: Role,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"INSERT INTO "user" (login, role)
           VALUES ($1, $2)
           ON CONFLICT (login) DO NOTHING
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        login,
        role as _
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert_guest(pool: &PgPool, login: &str) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"INSERT INTO "user" (login, guest)
           VALUES ($1, true)
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        login
    )
    .fetch_one(pool)
    .await
}

/// Turn a guest into a password account, in place. `None` when the row is not (or no longer) a guest.
pub async fn convert_guest_with_password(
    pool: &PgPool,
    user_id: Uuid,
    login: &str,
    password_hash: &str,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"UPDATE "user"
           SET login = $2, password_hash = $3, guest = false
           WHERE id = $1 AND guest
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        user_id,
        login,
        password_hash
    )
    .fetch_optional(pool)
    .await
}

/// The account holding a provider id: pass the id in its provider's slot and `None` in the other.
pub async fn find_by_provider_id(
    pool: &PgPool,
    discord_id: Option<&str>,
    google_id: Option<&str>,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"SELECT id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at
           FROM "user"
           WHERE discord_id = $1 OR google_id = $2"#,
        discord_id,
        google_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_by_email_hash(pool: &PgPool, email_hash: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"SELECT id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at
           FROM "user"
           WHERE email_hash = $1"#,
        email_hash
    )
    .fetch_optional(pool)
    .await
}

/// Link a provider id to an account whose slot for it is still empty; `None` when it no longer is.
pub async fn link_provider(
    pool: &PgPool,
    user_id: Uuid,
    discord_id: Option<&str>,
    google_id: Option<&str>,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"UPDATE "user"
           SET discord_id = COALESCE($2, discord_id), google_id = COALESCE($3, google_id)
           WHERE id = $1
             AND ($2::text IS NULL OR discord_id IS NULL)
             AND ($3::text IS NULL OR google_id IS NULL)
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        user_id,
        discord_id,
        google_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn set_email_hash(
    pool: &PgPool,
    user_id: Uuid,
    email_hash: &str,
    version: i16,
) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"UPDATE "user"
           SET email_hash = $2, email_hash_version = $3
           WHERE id = $1
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        user_id,
        email_hash,
        version
    )
    .fetch_one(pool)
    .await
}

/// Turn a guest into a provider account, in place and under its generated login.
/// `None` when the row is not (or no longer) a guest.
pub async fn convert_guest_with_provider(
    pool: &PgPool,
    user_id: Uuid,
    discord_id: Option<&str>,
    google_id: Option<&str>,
    email_hash: Option<&str>,
    email_hash_version: Option<i16>,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"UPDATE "user"
           SET discord_id = $2, google_id = $3, email_hash = $4, email_hash_version = $5, guest = false
           WHERE id = $1 AND guest
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        user_id,
        discord_id,
        google_id,
        email_hash,
        email_hash_version
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert_with_provider(
    pool: &PgPool,
    login: &str,
    discord_id: Option<&str>,
    google_id: Option<&str>,
    email_hash: Option<&str>,
    email_hash_version: Option<i16>,
) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"INSERT INTO "user" (login, discord_id, google_id, email_hash, email_hash_version)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        login,
        discord_id,
        google_id,
        email_hash,
        email_hash_version
    )
    .fetch_one(pool)
    .await
}

pub async fn touch_last_login(pool: &PgPool, user_id: Uuid) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"UPDATE "user"
           SET last_login_date = now()
           WHERE id = $1
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        user_id
    )
    .fetch_one(pool)
    .await
}

pub async fn set_role(pool: &PgPool, user_id: Uuid, role: Role) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"UPDATE "user"
           SET role = $2
           WHERE id = $1
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        user_id,
        role as _
    )
    .fetch_one(pool)
    .await
}

pub async fn set_login(pool: &PgPool, user_id: Uuid, login: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"UPDATE "user"
           SET login = $2
           WHERE id = $1
           RETURNING id, login, role as "role: Role", password_hash, guest, discord_id, google_id, email_hash, email_hash_version, last_login_date, created_at, updated_at"#,
        user_id,
        login
    )
    .fetch_optional(pool)
    .await
}
