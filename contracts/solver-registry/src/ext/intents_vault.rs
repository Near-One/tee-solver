use near_sdk::{ext_contract, json_types::U128, AccountId, PromiseOrValue, PublicKey};

#[allow(dead_code)]
#[ext_contract(ext_intents_vault)]
trait IntentsVaultContract {
    fn add_public_key(intents_contract_id: AccountId, public_key: PublicKey);
    fn remove_public_key(intents_contract_id: AccountId, public_key: PublicKey);
    fn ft_withdraw(
        intents_contract_id: AccountId,
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: Option<String>,
    ) -> PromiseOrValue<U128>;
}
