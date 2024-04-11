use claim_model::{
    account_record::{AccountRecord, AccountRecordVersioned},
    Duration, TokensAmount, UnixTimestamp,
};
use near_sdk::{
    env::{block_timestamp_ms, panic_str},
    store::LookupMap,
    AccountId,
};

mod asserts;
pub(crate) mod tests;

fn ms_timestamp_to_seconds(ms: u64) -> UnixTimestamp {
    u32::try_from(ms / 1000)
        .unwrap_or_else(|err| panic_str(&format!("Failed to get convert milliseconds to Unix timestamp: {err}")))
}

pub(crate) fn now_seconds() -> UnixTimestamp {
    ms_timestamp_to_seconds(block_timestamp_ms())
}

pub(crate) trait UnixTimestampExtension {
    fn is_within_period(&self, now: UnixTimestamp, period: Duration) -> bool;
}

impl UnixTimestampExtension for UnixTimestamp {
    fn is_within_period(&self, now: UnixTimestamp, period: Duration) -> bool {
        now - self < period
    }
}

#[test]
fn convert_milliseconds_to_unix_timestamp_successfully() {
    let millis: u64 = 1_699_038_575_819;
    let timestamp = ms_timestamp_to_seconds(millis);

    assert_eq!(1_699_038_575, timestamp);
}

#[test]
#[should_panic(expected = "Failed to get convert milliseconds to Unix timestamp")]
fn convert_milliseconds_to_unix_timestamp_with_unsuccessfully() {
    let millis: u64 = u64::MAX;
    let _timestamp = ms_timestamp_to_seconds(millis);
}

pub type AccountMap = LookupMap<AccountId, AccountRecordVersioned>;

pub(crate) trait AccountAccessor {
    fn get_account(&self, account_id: &AccountId) -> &AccountRecord;

    fn get_account_mut(&mut self, account_id: &AccountId) -> &mut AccountRecord;
}

impl AccountAccessor for AccountMap {
    fn get_account(&self, account_id: &AccountId) -> &AccountRecord {
        let AccountRecordVersioned::V1(account) = self.get(account_id).expect("Account not found");
        account
    }

    fn get_account_mut(&mut self, account_id: &AccountId) -> &mut AccountRecord {
        if !self.contains_key(account_id) {
            self.insert(account_id.clone(), AccountRecordVersioned::new(now_seconds()));
        }

        let AccountRecordVersioned::V1(account) = self.get_mut(account_id).expect("Account not found");
        account
    }
}
