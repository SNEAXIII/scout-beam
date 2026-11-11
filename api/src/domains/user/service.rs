//! The rules of the user domain: a pool plus plain values in, `Result<_, UserError>` out.

use sqlx::PgPool;
use uuid::Uuid;

use crate::domains::user::error::UserError;
use crate::domains::user::login_generator;
use crate::domains::user::model::User;
use crate::domains::user::repository;
use crate::shared::role::Role;

/// Load the account behind a token: "the token decodes" does not prove the row still exists.
pub async fn load_active(pool: &PgPool, user_id: Uuid) -> Result<User, UserError> {
    repository::find_by_id(pool, user_id)
        .await?
        .ok_or(UserError::DoesNotExist)
}

/// Find the account for a login, creating it on first sight.
pub async fn login_or_create(
    pool: &PgPool,
    login: Option<&str>,
    role: Option<Role>,
) -> Result<User, UserError> {
    // A named account is looked up first, so a second call signs back into it.
    let user = if let Some(login) = login {
        match repository::find_by_login(pool, login).await? {
            Some(existing) => existing,
            None => {
                match repository::insert_if_absent(pool, login, role.unwrap_or(Role::User)).await? {
                    Some(created) => created,
                    // Two first sign-ins raced; both asked to sign in, so both do.
                    None => repository::find_by_login(pool, login)
                        .await?
                        .ok_or(UserError::DoesNotExist)?,
                }
            }
        }
    } else {
        let generated =
            login_generator::unique_login(pool, &login_generator::random_base()).await?;
        repository::insert(pool, &generated, role.unwrap_or(Role::User))
            .await
            .map_err(UserError::from_write)?
    };

    // Not atomic with the creation: a failure in between leaves a null `last_login_date`, which no rule reads.
    repository::touch_last_login(pool, user.id).await?;
    Ok(user)
}

/// What `seed` actually did, so a caller can report it instead of guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedOutcome {
    Created,
    RoleChanged { from: Role },
    Unchanged,
}

/// Create an account, or align an existing one on the wanted role.
/// The tooling door, used by the `seed_user` binary and never by a route.
pub async fn seed(
    pool: &PgPool,
    login: &str,
    role: Role,
) -> Result<(User, SeedOutcome), UserError> {
    validate_login(login)?;

    let Some(existing) = repository::find_by_login(pool, login).await? else {
        let created = repository::insert(pool, login, role)
            .await
            .map_err(UserError::from_write)?;
        return Ok((created, SeedOutcome::Created));
    };

    if existing.role == role {
        return Ok((existing, SeedOutcome::Unchanged));
    }

    let previous = existing.role;
    let updated = repository::set_role(pool, existing.id, role).await?;

    Ok((updated, SeedOutcome::RoleChanged { from: previous }))
}

/// The DTO's rule, applied before the CHECK constraint can turn a typo into an opaque error.
fn validate_login(login: &str) -> Result<(), UserError> {
    let length = login.chars().count();
    let alphanumeric = login.chars().all(|c| c.is_ascii_alphanumeric());

    if (3..=30).contains(&length) && alphanumeric {
        Ok(())
    } else {
        Err(UserError::InvalidLogin)
    }
}
