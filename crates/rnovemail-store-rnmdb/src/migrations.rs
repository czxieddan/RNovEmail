use crate::RNOVEMAIL_NAMESPACES;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationPlan {
    namespaces: Vec<&'static str>,
}

impl MigrationPlan {
    pub fn current() -> Self {
        Self {
            namespaces: RNOVEMAIL_NAMESPACES.to_vec(),
        }
    }

    pub fn namespaces(&self) -> &[&'static str] {
        &self.namespaces
    }
}
