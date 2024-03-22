#![cfg(test)]

use claim_model::{
    api::{AssetsApi, ClaimApi, ClaimTotalApi, RecordApi},
    Asset, ClaimAvailabilityView, ClaimResultView, UnixTimestamp,
};
use near_sdk::{json_types::U128, AccountId, PromiseOrValue};

use crate::{
    claim::api::single::test::EXT_TRANSFER_FUTURE,
    common::{
        tests::{data::set_test_future_success, Context},
        AssetExt,
    }, get_default_asset,
};

#[test]
fn test_check_claim_availability_when_user_is_not_registered() {
    let (_, contract, accounts) = Context::init_with_oracle();

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance);

    let alice_can_claim = contract.is_claim_available(accounts.alice);
    assert_eq!(ClaimAvailabilityView::Unregistered, alice_can_claim);
}

#[test]
fn test_check_claim_availability_when_user_has_tokens_and_claim_period_after_claim_is_not_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance = 400_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(alice_balance, alice_new_balance);

    let claim_timestamp = contract.claim_period as u64 + 100;
    context.set_block_timestamp_in_seconds(claim_timestamp);
    context.switch_account(&accounts.alice);
    contract.claim();

    let check_timestamp = claim_timestamp + 10;
    context.set_block_timestamp_in_seconds(check_timestamp);

    let alice_can_claim = contract.is_claim_available(accounts.alice.clone());
    assert_eq!(
        alice_can_claim,
        ClaimAvailabilityView::Unavailable((claim_timestamp as UnixTimestamp, contract.claim_period))
    );
}

#[test]
fn test_check_claim_availability_when_user_has_tokens_and_claim_period_after_claim_is_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance = 300_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(alice_balance, alice_new_balance);

    let claim_timestamp = contract.claim_period as u64 + 100;
    context.set_block_timestamp_in_seconds(claim_timestamp);
    context.switch_account(&accounts.alice);
    contract.claim();

    let check_timestamp = claim_timestamp + contract.claim_period as u64 + 100;
    context.set_block_timestamp_in_seconds(check_timestamp);

    let alice_can_claim = contract.is_claim_available(accounts.alice.clone());
    assert_eq!(alice_can_claim, ClaimAvailabilityView::Available);
}

#[test]
fn test_check_claim_availability_when_user_has_tokens_and_claim_period_after_record_creation_is_not_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance = 400_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(alice_balance, alice_new_balance);

    let alice_can_claim = contract.is_claim_available(accounts.alice.clone());
    assert_eq!(
        alice_can_claim,
        ClaimAvailabilityView::Unavailable((0, contract.claim_period))
    );
}

#[test]
fn test_check_claim_availability_when_user_has_tokens_and_claim_period_after_record_creation_is_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance = 300_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.set_block_timestamp_in_seconds(contract.claim_period as u64 + 100);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(alice_balance, alice_new_balance);

    let alice_can_claim = contract.is_claim_available(accounts.alice.clone());
    assert_eq!(alice_can_claim, ClaimAvailabilityView::Available);
}

#[test]
#[should_panic(expected = "Claim is not available at the moment")]
fn test_claim_when_user_is_not_registered() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance);

    context.switch_account(&accounts.alice);
    contract.claim();
}

#[test]
#[should_panic(expected = "Claim is not available at the moment")]
fn test_claim_when_user_has_tokens_and_claim_period_is_not_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    let alice_balance = 200_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.switch_account(&accounts.alice);
    contract.claim();
}

#[test]
fn test_claim_when_user_has_tokens_and_current_time_matches_claim_period() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance = 500_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.set_block_timestamp_in_seconds(contract.burn_period as u64);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance);

    let alice_can_claim = contract.is_claim_available(accounts.alice.clone());
    assert_eq!(alice_can_claim, ClaimAvailabilityView::Available);
}

#[test]
fn test_claim_when_user_has_tokens_and_claim_period_is_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    let alice_balance = 700_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.set_block_timestamp_in_seconds(contract.claim_period as u64 + 100);

    context.switch_account(&accounts.alice);
    let claimed_amount = match contract.claim() {
        PromiseOrValue::Promise(_) => panic!("Expected value"),
        PromiseOrValue::Value(value) => value,
    };
    assert_eq!(alice_balance, claimed_amount.total.unwrap().0);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance);
}

#[test]
fn test_claim_when_user_has_tokens_and_burn_period_is_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    let alice_balance = 12_000_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.set_block_timestamp_in_seconds(contract.burn_period as u64 + 100);

    context.switch_account(&accounts.alice);
    let claimed_amount = match contract.claim() {
        PromiseOrValue::Promise(_) => panic!("Expected value"),
        PromiseOrValue::Value(value) => value,
    };
    assert_eq!(0, claimed_amount.total.unwrap().0);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance);
}

#[test]
fn test_claim_when_user_has_tokens_and_claim_period_is_passed_and_transfer_failed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, false);

    let alice_balance = 123_100_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.set_block_timestamp_in_seconds(contract.claim_period as u64 + 100);

    context.switch_account(&accounts.alice);
    let claimed_amount = match contract.claim() {
        PromiseOrValue::Promise(_) => panic!("Expected value"),
        PromiseOrValue::Value(value) => value,
    };
    assert_eq!(None, claimed_amount.total);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(alice_balance, alice_new_balance);
}

#[test]
fn test_claim_when_user_has_claimable_balance_for_multiple_tokens() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    let alice_default_token_balance = 1_000_000;
    let alice_extra_token_balance = 50_000_000;

    let extra_asset: Asset = "USDT".to_string();
    let extra_asset_contract_id = AccountId::new_unchecked("usdt.testnet".to_string());

    context.switch_account(&accounts.oracle);

    contract.register_asset(extra_asset.clone(), extra_asset_contract_id);

    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_default_token_balance))], None);
    contract.record_batch_for_hold(
        vec![(accounts.alice.clone(), U128(alice_extra_token_balance))],
        Some(extra_asset.clone()),
    );

    let alice_default_token_new_balance = contract.get_claimable_balance_for_account(accounts.alice.clone(), None);
    assert_eq!(alice_default_token_new_balance.0, alice_default_token_balance);

    let alice_extra_token_new_balance =
        contract.get_claimable_balance_for_account(accounts.alice.clone(), Some(extra_asset.clone()));
    assert_eq!(alice_extra_token_new_balance.0, alice_extra_token_balance);

    context.set_block_timestamp_in_seconds((contract.claim_period + 1) as _);

    context.switch_account(&accounts.alice);
    let PromiseOrValue::Value(claim_result) = contract.claim_all() else {
        panic!("Expected value");
    };
    assert_eq!(2, claim_result.len());

    assert!(claim_result.contains(&ClaimResultView::new(
        get_default_asset().normalize(),
        true,
        Some(alice_default_token_balance),
    )));

    assert!(claim_result.contains(&ClaimResultView::new(
        extra_asset.normalize(),
        true,
        Some(alice_extra_token_balance),
    )));
}
