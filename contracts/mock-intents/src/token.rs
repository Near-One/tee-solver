use std::str::FromStr;

use near_sdk::{near, AccountId};

#[near(serializers = [json, borsh])]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenId {
    account_id: AccountId,
    standard: String,
}

impl TokenId {
    pub fn new(account_id: AccountId, standard: String) -> Self {
        Self {
            account_id,
            standard,
        }
    }

    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    pub fn standard(&self) -> &String {
        &self.standard
    }
}

impl From<AccountId> for TokenId {
    fn from(account_id: AccountId) -> Self {
        Self {
            account_id,
            standard: "nep141".to_string(),
        }
    }
}

impl FromStr for TokenId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (standard, account_id) = s.split_once(':').ok_or("Invalid token id")?;
        Ok(Self {
            account_id: account_id.parse().map_err(|_| "Invalid account id")?,
            standard: standard.to_string(),
        })
    }
}

impl std::fmt::Display for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.standard, self.account_id)
    }
}

impl std::fmt::Debug for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.standard, self.account_id)
    }
}
