use near_sdk::{near_bindgen, AccountId};

use crate::*;

pub trait CleanApi {
    fn clean(&mut self, account_ids: Vec<AccountId>);
}

#[near_bindgen]
impl CleanApi for Contract {
    fn clean(&mut self, account_ids: Vec<AccountId>) {
        self.assert_oracle();

        for account_id in account_ids {
            self.accounts.set(account_id, None);
        }
    }
}
