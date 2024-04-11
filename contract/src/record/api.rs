use claim_model::{
    api::RecordApi,
    event::{emit, EventKind::Record, RecordAmountDetailed, RecordData},
    TokensAmount,
};
use near_sdk::{json_types::U128, near_bindgen, AccountId};

use crate::{
    common::{now_seconds, AccountAccessor, UnixTimestampExtension},
    Contract, ContractExt,
};

#[near_bindgen]
impl RecordApi for Contract {
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) {
        self.assert_oracle();

        let now_seconds = now_seconds();
        let claimable_window_start = now_seconds - self.burn_period;
        let mut event_data = RecordData::new(now_seconds);

        for (account_id, amount) in amounts {
            self.migrate_account_if_outdated(&account_id);

            let account = self.accounts.get_account_mut(&account_id);
            let mut balance_to_burn: TokensAmount = 0;

            if !account
                .claim_period_refreshed_at
                .is_within_period(now_seconds, self.burn_period)
            {
                balance_to_burn = account.burn_rate * (claimable_window_start - account.last_burn_at) as u128;
                self.balance_to_burn += balance_to_burn;
                account.balance -= balance_to_burn;
            }

            account.balance += amount.0;
            account.last_top_up_at = now_seconds;

            account.burn_rate = account.balance / self.burn_period as u128;
            account.last_burn_at = now_seconds;

            event_data.amounts.push((
                account_id.clone(),
                RecordAmountDetailed {
                    credited: amount,
                    burnt: U128(balance_to_burn),
                },
            ));
        }

        emit(Record(event_data));
    }
}
