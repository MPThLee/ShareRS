pub mod models;
mod postgres;

pub use postgres::{check_for_migrations, connect};
