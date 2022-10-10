use crate::multi_token::core::MultiToken;
use crate::storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement};
use near_sdk::json_types::U128;
use near_sdk::{assert_one_yocto, env, log, AccountId, Balance, Promise};

impl MultiToken {
    /// Internal method that returns the Account ID and the balance in case the account was
    /// unregistered.
    pub fn internal_storage_unregister(
        &mut self,
        force: Option<bool>,
    ) -> Option<(AccountId, Balance)> {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let force = force.unwrap_or(false);

        // todo: discuss it
        if force {
            env::panic_str("Force is not supported for MultiToken");
        }

        let tokens_amount = self.get_tokens_amount(&account_id);

        let storage_balance = self.accounts_storage.get(&account_id);
        if storage_balance.is_none() {
            log!("The account {} is not registered", &account_id);
            return None;
        }

        if tokens_amount == 0 {
            self.accounts_storage.remove(&account_id);
            let balance = storage_balance.unwrap();
            Promise::new(account_id.clone()).transfer(balance);
            Some((account_id, balance))
        } else {
            env::panic_str(
                "Can't unregister the account with the positive amount of tokens without force",
            )
        }
    }

    fn internal_storage_balance_of(&self, account_id: &AccountId) -> Option<StorageBalance> {
        if self.accounts_storage.contains_key(account_id) {
            Some(StorageBalance { total: self.storage_balance_bounds().min, available: 0.into() })
        } else {
            None
        }
    }

    fn storage_cost(&self, account_id: &AccountId) -> Balance {
        if let Some(tokens) = &self.tokens_per_owner {
            if let Some(user_tokens) = tokens.get(account_id) {
                return (user_tokens.len() * self.storage_usage_per_token + self.account_storage_usage)
                    as Balance * env::storage_byte_cost();
            }
        }

        (self.account_storage_usage + self.storage_usage_per_token) as Balance * env::storage_byte_cost()
    }

    fn get_tokens_amount(&self, account_id: &AccountId) -> u64 {
        if let Some(tokens) = &self.tokens_per_owner {
            if let Some(user_tokens) = tokens.get(account_id) {
                return user_tokens.len();
            }
        }

        0
    }

    pub fn assert_storage_usage(&self, account_id: &AccountId) {
        let storage_cost = self.storage_cost(account_id);
        let storage_balance = self.accounts_storage.get(account_id);
        if let Some(balance) = storage_balance {
            if balance < storage_cost {
                env::panic_str(format!("The account doesn't have enough storage balance. Balance {}, required {}",
                                       balance, storage_cost).as_str());
            }
        } else {
            env::panic_str("The account is not registered");
        }
    }
}

impl StorageManagement for MultiToken {
    #[allow(unused_variables)]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount: Balance = env::attached_deposit();
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        if self.accounts_storage.contains_key(&account_id) && registration_only.is_some() {
            log!("The account is already registered, refunding the deposit");
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            let min_balance: u128 = self.storage_balance_bounds().min.into();
            if amount < min_balance {
                env::panic_str("The attached deposit is less than the minimum storage balance");
            }

            let current_amount = self.accounts_storage.get(&account_id).unwrap_or(0);
            self.accounts_storage.insert(&account_id, &(amount + current_amount));
        }
        self.internal_storage_balance_of(&account_id).unwrap()
    }

    /// While storage_withdraw normally allows the caller to retrieve `available` balance, the basic
    /// Fungible Token implementation sets storage_balance_bounds.min == storage_balance_bounds.max,
    /// which means available balance will always be 0. So this implementation:
    /// * panics if `amount > 0`
    /// * never transfers Ⓝ to caller
    /// * returns a `storage_balance` struct if `amount` is 0
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        // todo: we should decrease accounts_storage
        let predecessor_account_id = env::predecessor_account_id();
        if let Some(storage_balance) = self.internal_storage_balance_of(&predecessor_account_id) {
            match amount {
                Some(amount) if amount.0 > 0 => {
                    env::panic_str("The amount is greater than the available storage balance");
                }
                _ => storage_balance,
            }
        } else {
            env::panic_str(
                format!("The account {} is not registered", &predecessor_account_id).as_str(),
            );
        }
    }

    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        self.internal_storage_unregister(force).is_some()
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let required_storage_balance =
            Balance::from(self.account_storage_usage) * env::storage_byte_cost()
                + Balance::from(self.storage_usage_per_token) * env::storage_byte_cost();
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            // The max amount of storage is unlimited, because we don't know the amount of tokens
            max: None,
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.accounts_storage.get(&account_id).map(|account_balance| {
            StorageBalance {
                total: account_balance.into(),
                available: account_balance.saturating_sub(self.storage_cost(&account_id)).into(),
            }
        })
    }
}
