//! Business domains: one folder each, owning its model, SQL, rules, errors and routes.
//!
//! A domain may use `shared` and `auth`; it never reaches into another domain's repository.

pub mod user;
