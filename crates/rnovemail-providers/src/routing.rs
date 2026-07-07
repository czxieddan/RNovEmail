use rnovemail_domain::ProviderAccount;

use crate::{ProviderError, SendMailRequest};

#[derive(Clone, Debug, Default)]
pub struct ProviderRegistry {
    accounts: Vec<ProviderAccount>,
}

impl ProviderRegistry {
    pub fn new(accounts: impl IntoIterator<Item = ProviderAccount>) -> Self {
        Self {
            accounts: accounts.into_iter().collect(),
        }
    }

    pub fn select_account(
        &self,
        request: &SendMailRequest,
    ) -> Result<&ProviderAccount, ProviderError> {
        let domain = request.from().domain();
        self.accounts
            .iter()
            .find(|account| account.serves_domain(domain))
            .ok_or(ProviderError::NoProviderForDomain)
    }
}
