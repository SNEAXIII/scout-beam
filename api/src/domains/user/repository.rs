//! Every SQL statement of the user domain, and nothing else.
//!
//! `query_as!` checks each one against a live schema at compile time.

use sqlx::PgPool;
use uuid::Uuid;

use crate::domains::user::model::User;
use crate::shared::role::Role;

pub async fn find_by_id(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"SELECT id, login, role as "role: Role", last_login_date, created_at, updated_at
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
        r#"SELECT id, login, role as "role: Role", last_login_date, created_at, updated_at
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
           RETURNING id, login, role as "role: Role", last_login_date, created_at, updated_at"#,
        login,
        role as _
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
           RETURNING id, login, role as "role: Role", last_login_date, created_at, updated_at"#,
        login,
        role as _
    )
    .fetch_optional(pool)
    .await
}

pub async fn touch_last_login(pool: &PgPool, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        r#"UPDATE "user" SET last_login_date = now() WHERE id = $1"#,
        user_id
    )
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn set_role(pool: &PgPool, user_id: Uuid, role: Role) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"UPDATE "user"
           SET role = $2
           WHERE id = $1
           RETURNING id, login, role as "role: Role", last_login_date, created_at, updated_at"#,
        user_id,
        role as _
    )
    .fetch_one(pool)
    .await
}
