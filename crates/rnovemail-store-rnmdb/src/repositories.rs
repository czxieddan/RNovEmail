use std::path::{Path, PathBuf};

use crate::MigrationPlan;

#[derive(Clone, Debug)]
pub struct RnovStore {
    data_dir: PathBuf,
    migrations: MigrationPlan,
}

impl RnovStore {
    pub fn open(data_dir: impl AsRef<Path>) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            migrations: MigrationPlan::current(),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn migrations(&self) -> &MigrationPlan {
        &self.migrations
    }
}
