use crate::*;
use near_sdk::{json_types::U128, near, PromiseOrValue};

#[near]
impl Contract {
    /// Approve a docker compose hash for worker registration
    #[payable]
    pub fn approve_compose_hash(&mut self, compose_hash: String) {
        self.assert_owner();
        DockerComposeHash::try_from_hex(compose_hash.clone()).expect("Invalid compose hash");

        self.approved_compose_hashes.insert(compose_hash.clone());

        Event::ComposeHashApproved {
            compose_hash: &compose_hash,
        }
        .emit();
    }

    /// Remove an approved docker compose hash
    #[payable]
    pub fn remove_compose_hash(&mut self, compose_hash: String) {
        self.assert_owner();
        DockerComposeHash::try_from_hex(compose_hash.clone()).expect("Invalid compose hash");

        require!(
            self.approved_compose_hashes.remove(&compose_hash),
            "Compose hash not found"
        );

        Event::ComposeHashRemoved {
            compose_hash: &compose_hash,
        }
        .emit();
    }

    #[payable]
    pub fn change_owner(&mut self, new_owner_id: AccountId) {
        self.assert_owner();
        let old_owner_id = self.owner_id.clone();
        self.owner_id = new_owner_id.clone();

        Event::OwnerChanged {
            old_owner_id: &old_owner_id,
            new_owner_id: &new_owner_id,
        }
        .emit();
    }

    /// Withdraw tokens from a pool to owner account
    #[payable]
    pub fn withdraw_from_pool(
        &mut self,
        pool_id: u32,
        token_id: AccountId,
        amount: U128,
    ) -> PromiseOrValue<U128> {
        self.assert_owner();

        let pool = self.pools.get(pool_id).expect("Pool not found");
        require!(pool.token_ids.contains(&token_id), "Invalid token ID");
        // We do not check that the amount does not exceed the pool balance because the solver registry
        // is not tracking the real-time NEAR Intents balance of the pool
        require!(amount.0 > 0, "Invalid amount");

        let pool_account_id = self.get_pool_account_id(pool_id);
        ext_intents_vault::ext(pool_account_id.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_withdraw(
                self.intents_contract_id.clone(),
                token_id.clone(),
                self.owner_id.clone(),
                amount,
                None,
                None,
            )
            .into()
    }
}

impl Contract {
    pub(crate) fn assert_owner(&self) {
        assert_one_yocto();
        require!(env::predecessor_account_id() == self.owner_id);
    }
}
