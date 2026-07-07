mod codec;
mod keys;
mod migrations;
mod repositories;
mod schema;

pub use codec::{decode, encode};
pub use keys::namespace_key;
pub use migrations::MigrationPlan;
pub use repositories::{RnovStore, RnovStoreKey};
pub use schema::RNOVEMAIL_NAMESPACES;

pub fn embedded_rnov_packages() -> [&'static str; 4] {
    [
        "rnmdb-common",
        "rnmdb-storage",
        "rnmdb-security",
        "rnmdb-instance",
    ]
}
