//  Copyright 2021 PolyCrypt GmbH
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

#![allow(dead_code)]

use super::mock::*;

use codec::Encode;
use frame_support::{
	assert_ok,
	traits::{OnFinalize, OnInitialize},
};
use pallet_perun::types::{ChannelIdOf, FundingIdOf, SecondsOf, SigOf, StateOf, WithdrawalOf};
use sp_core::{crypto::*, H256};

/// Checks that the last event was a `Deposited` event with the given args.
pub fn event_deposited(funding_id: H256, amount: u64) {
	assert_eq!(
		last_event(),
		Event::Perun(pallet_perun::Event::Deposited(funding_id, amount))
	);
}

/// Checks that the last event was a `Disputed` event with the given args.
pub fn event_disputed(channel_id: ChannelIdOf<Test>, state: StateOf<Test>) {
	assert_eq!(
		last_event(),
		Event::Perun(pallet_perun::Event::Disputed(channel_id, state))
	);
}

/// Checks that the last event was a `Concluded` event with the given args.
pub fn event_concluded(channel_id: ChannelIdOf<Test>) {
	assert_eq!(
		last_event(),
		Event::Perun(pallet_perun::Event::Concluded(channel_id))
	);
}

/// Checks that the last event was a `Withdrawn` event with the given args.
pub fn event_withdrawn(funding_id: FundingIdOf<Test>) {
	assert_eq!(
		last_event(),
		Event::Perun(pallet_perun::Event::Withdrawn(funding_id))
	);
}

/// Returns the last events.
/// Panics in case that there is none.
pub fn last_event() -> Event {
	System::events().pop().expect("Event list empty").event
}

/// Asserts that no event was emitted.
pub fn assert_no_events() {
	assert!(System::events().is_empty());
}

/// The number of events that were emitted.
pub fn num_events() -> usize {
	System::events().len()
}

/// Asserts that exactly `num` events were emitted.
pub fn assert_num_event(num: usize) {
	assert_eq!(num, System::events().len());
}

/// Increments the time by `sec` seconds.
pub fn increment_time(sec: SecondsOf<Test>) {
	for _ in 0..sec {
		let mut block = System::block_number();

		Perun::on_finalize(block);
		System::on_finalize(block);
		block += 1;
		System::set_block_number(block);

		System::on_initialize(block);
		Perun::on_initialize(block);
		// Set the time in ms.
		pallet_timestamp::Now::<Test>::set(block * 1000);
	}
}

/// Calls `Dispute` with the passed `setup` configuration.
/// `finalized` can be used to modify the state.
pub fn call_dispute(setup: &Setup, finalized: bool) -> StateOf<Test> {
	let mut state = setup.state.clone();
	state.finalized = finalized;
	let sigs = sign_state(&state, &setup);
	assert_ok!(Perun::dispute(
		Origin::signed(setup.ids.carl),
		setup.params.clone(),
		state.clone(),
		sigs
	));
	state
}

/// Creates off-chain signatures for `state` with alice and bob.
pub fn sign_state(state: &StateOf<Test>, setup: &Setup) -> Vec<SigOf<Test>> {
	let raw = Encode::encode(&state);
	let sig_alice = setup.keys.alice.sign(&raw);
	let sig_bob = setup.keys.bob.sign(&raw);
	vec![sig_alice, sig_bob]
}

/// Creates off-chain signatures for `withdrawal` with alice and bob.
pub fn sign_withdrawal(withdrawal: &WithdrawalOf<Test>, setup: &Setup) -> Vec<SigOf<Test>> {
	let raw = Encode::encode(&withdrawal);
	let sig_alice = setup.keys.alice.sign(&raw);
	let sig_bob = setup.keys.bob.sign(&raw);
	vec![sig_alice, sig_bob]
}

pub fn deposit_both(setup: &Setup) {
	assert_ok!(Perun::deposit(
		Origin::signed(setup.ids.alice),
		setup.fids.alice,
		setup.state.balances[0]
	));
	assert_ok!(Perun::deposit(
		Origin::signed(setup.ids.bob),
		setup.fids.bob,
		setup.state.balances[1]
	));
}
