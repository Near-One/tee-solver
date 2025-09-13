use near_sdk::store::LookupMap;
use near_sdk::{
    assert_one_yocto, env, ext_contract, json_types::U128, near, AccountId, BorshStorageKey,
    NearToken, PanicOnDefault, PromiseError, PromiseOrValue, PublicKey,
};
use std::collections::HashSet;

mod token;
mod token_receiver;

type Balance = u128;

use crate::token::TokenId;

#[allow(dead_code)]
#[ext_contract(ext_ft)]
trait FungibleTokenContract {
    fn ft_transfer(
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> PromiseOrValue<U128>;
}

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    public_keys: LookupMap<AccountId, HashSet<PublicKey>>,
    mt_balances: LookupMap<TokenId, LookupMap<AccountId, Balance>>,
}

#[near]
#[derive(BorshStorageKey)]
pub enum Prefix {
    PublicKeys,
    MultiTokenBalances,
    MultiTokenBalancesByTokenId(TokenId),
}

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn new() -> Self {
        Self {
            public_keys: LookupMap::new(Prefix::PublicKeys),
            mt_balances: LookupMap::new(Prefix::MultiTokenBalances),
        }
    }

    #[payable]
    pub fn add_public_key(&mut self, public_key: PublicKey) {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();
        let mut keys = self.internal_get_account(&account_id);
        keys.insert(public_key);
        self.public_keys.insert(account_id, keys.clone());
    }

    #[payable]
    pub fn remove_public_key(&mut self, public_key: PublicKey) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let mut keys = self.internal_get_account(&account_id);
        keys.remove(&public_key);
        self.public_keys.insert(account_id, keys.clone());
    }

    #[payable]
    pub fn ft_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        _msg: Option<String>,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        self.internal_withdraw_mt_balance(&receiver_id.clone(), &token.clone().into(), amount.0);

        ext_ft::ext(token)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer(receiver_id, amount, memo)
            .then(Self::ext(env::current_account_id()).on_ft_withdraw(amount))
            .into()
    }

    #[private]
    pub fn on_ft_withdraw(
        &mut self,
        amount: U128,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> U128 {
        if call_result.is_ok() {
            U128(amount.0)
        } else {
            U128(0)
        }
    }

    pub fn public_keys_of(&self, account_id: AccountId) -> HashSet<PublicKey> {
        self.internal_get_account(&account_id)
    }

    pub fn mt_batch_balance_of(&self, account_id: AccountId, token_ids: Vec<TokenId>) -> Vec<U128> {
        token_ids
            .into_iter()
            .map(|token_id| self.internal_mt_balance_of(&account_id, &token_id))
            .map(U128)
            .collect()
    }
}

impl Contract {
    fn internal_get_account(&self, account_id: &AccountId) -> HashSet<PublicKey> {
        self.public_keys
            .get(account_id)
            .unwrap_or(&HashSet::new())
            .clone()
    }

    fn internal_get_mt_balances(
        &mut self,
        token_id: &TokenId,
    ) -> &mut LookupMap<AccountId, Balance> {
        if !self.mt_balances.contains_key(token_id) {
            self.mt_balances.insert(
                token_id.clone(),
                LookupMap::new(Prefix::MultiTokenBalancesByTokenId(token_id.clone())),
            );
        }
        self.mt_balances.get_mut(token_id).unwrap()
    }

    fn internal_deposit_mt_balance(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
        amount: Balance,
    ) {
        let balances = self.internal_get_mt_balances(token_id);
        let current_balance = balances.get(account_id).unwrap_or(&0);
        balances.insert(account_id.clone(), current_balance + amount);
    }

    fn internal_withdraw_mt_balance(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
        amount: Balance,
    ) {
        let balances = self.internal_get_mt_balances(token_id);
        let current_balance = balances.get(account_id).unwrap_or(&0);
        if amount > *current_balance {
            env::panic_str("Insufficient balance for withdrawal");
        }
        balances.insert(account_id.clone(), current_balance - amount);
    }

    fn internal_mt_balance_of(&self, account_id: &AccountId, token_id: &TokenId) -> Balance {
        self.mt_balances
            .get(token_id)
            .and_then(|balances| balances.get(account_id))
            .unwrap_or(&0)
            .clone()
    }
}
