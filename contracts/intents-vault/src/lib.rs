use near_sdk::{
    assert_one_yocto, env, ext_contract, json_types::U128, near, require, AccountId, NearToken,
    Promise, PromiseOrValue, PublicKey,
};

#[allow(dead_code)]
#[ext_contract(ext_intents)]
trait IntentsContract {
    /// Registers or re-activates `public_key` under the caller account_id.
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    ///
    /// Referenced source code: https://github.com/near/intents/blob/42eb99382722bd29e4998fece820a5bc54b3fec1/defuse/src/accounts.rs#L19
    fn add_public_key(public_key: PublicKey);

    /// Deactivate `public_key` from the caller account_id,
    /// i.e. this key can't be used to make any actions unless it's re-created.
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    ///
    /// Referenced source code: https://github.com/near/intents/blob/42eb99382722bd29e4998fece820a5bc54b3fec1/defuse/src/accounts.rs#L25
    fn remove_public_key(public_key: PublicKey);

    /// Returns number of tokens were successfully withdrawn.
    ///
    /// Optionally can specify `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    ///
    /// Referenced source code: https://github.com/near/intents/blob/42eb99382722bd29e4998fece820a5bc54b3fec1/defuse/src/tokens/nep141.rs#L13
    fn ft_withdraw(
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: Option<String>,
    ) -> PromiseOrValue<U128>;
}

#[derive(Default)]
#[near(contract_state)]
pub struct Contract {}

/// TODO: the contract can be deployed as a global contract
#[near]
impl Contract {
    #[payable]
    pub fn add_public_key(
        &mut self,
        intents_contract_id: AccountId,
        public_key: PublicKey,
    ) -> Promise {
        assert_one_yocto();
        self.require_parent_account();

        ext_intents::ext(intents_contract_id)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .add_public_key(public_key)
    }

    #[payable]
    pub fn remove_public_key(
        &mut self,
        intents_contract_id: AccountId,
        public_key: PublicKey,
    ) -> Promise {
        assert_one_yocto();
        self.require_parent_account();

        ext_intents::ext(intents_contract_id)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .remove_public_key(public_key)
    }

    #[payable]
    pub fn ft_withdraw(
        &mut self,
        intents_contract_id: AccountId,
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: Option<String>,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        self.require_parent_account();

        ext_intents::ext(intents_contract_id)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_withdraw(token, receiver_id, amount, memo, msg)
            .into()
    }
}

impl Contract {
    fn require_parent_account(&self) {
        let contract_id = env::current_account_id().to_string();
        let (_, parent_account_id) = contract_id.split_once('.').expect("Invalid contract ID");
        require!(
            env::predecessor_account_id() == parent_account_id,
            "Only parent account can perform this action"
        );
    }
}
