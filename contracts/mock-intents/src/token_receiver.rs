use near_sdk::json_types::U128;
use near_sdk::{log, near, AccountId, PromiseOrValue};

use crate::*;

#[near]
impl Contract {
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let receiver_id = if msg.is_empty() {
            sender_id
        } else {
            msg.parse().unwrap()
        };

        let token_id: TokenId = env::predecessor_account_id().into();

        self.internal_deposit_mt_balance(&receiver_id.clone(), &token_id.clone(), amount.0);

        log!(
            "Deposit {} {} into intents contract for account {}",
            amount.0,
            token_id,
            receiver_id
        );

        PromiseOrValue::Value(U128(0))
    }
}
