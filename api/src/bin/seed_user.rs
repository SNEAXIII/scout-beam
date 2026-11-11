//! `seed-user` — create a development account, or promote an existing one.
//!
//! Runs the very same `service::seed`, with no router, no state and no HTTP.

use anyhow::Result;
use clap::{Parser, ValueEnum};
use scout_beam::Config;
use scout_beam::domains::user::service::{self, SeedOutcome};
use scout_beam::shared::db;
use scout_beam::shared::role::Role;

#[derive(Debug, Parser)]
#[command(name = "seed-user", about = "Create or promote a development account")]
struct Args {
    /// Account login: 3 to 30 letters and digits.
    #[arg(long)]
    login: String,

    /// Role to grant.
    #[arg(long, value_enum, default_value_t = RoleArg::User)]
    role: RoleArg,
}

/// Deriving `ValueEnum` on `Role` would drag clap into the server build, so the CLI keeps its own.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum RoleArg {
    User,
    Admin,
    SuperAdmin,
}

impl From<RoleArg> for Role {
    fn from(role: RoleArg) -> Self {
        match role {
            RoleArg::User => Self::User,
            RoleArg::Admin => Self::Admin,
            RoleArg::SuperAdmin => Self::SuperAdmin,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url).await?;

    let (user, outcome) = service::seed(&pool, &args.login, args.role.into()).await?;

    // Plain ASCII, no tracing setup: a command a human reads.
    match outcome {
        SeedOutcome::Created => println!(
            "[created] {} | role={} | id={}",
            user.login,
            user.role.as_str(),
            user.id
        ),
        SeedOutcome::RoleChanged { from } => println!(
            "[updated] {} | role {} -> {} | id={}",
            user.login,
            from.as_str(),
            user.role.as_str(),
            user.id
        ),
        SeedOutcome::Unchanged => println!(
            "[unchanged] {} | role={} | id={}",
            user.login,
            user.role.as_str(),
            user.id
        ),
    }

    println!(
        "[note]    sign in with: curl -X POST localhost:8002/auth/dev-login \
         -H 'content-type: application/json' -d '{{\"login\":\"{}\"}}'",
        user.login
    );

    Ok(())
}
