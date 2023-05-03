#[macro_use]
extern crate log;
use deadpool_postgres::{Pool as PGPool};

pub mod db;
pub mod error;
pub mod middleware;
pub mod routes;

const AUTH_EVENT_KIND: u64 = 27235;
const AUTH_EVENT_CREATED_AT_DELTA_SEC: u64 = 60; // Seconds

#[derive(Clone, Debug)]
pub struct AppData {
    pub pg_pool: PGPool,
    pub auth_event_host: String
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pubkey {
    pubkey: String,
}

impl Pubkey {
    pub fn into_inner(self) -> String {
        self.pubkey
    }
}
