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

mod common;
use common::mock::*;
use common::utils::*;

use frame_support::{assert_noop, assert_ok};
use pallet_perun::types::NonceOf;

#[test]
fn conclude_ok() {
	run_test(|setup| {
		let mut state = setup.state.clone();
		state.finalized = true;
		// Set the balances to 0 so it does not fail bc of missing deposits.
		state.balances = vec![0, 0];
		let sigs = sign_state(&state, &setup);

		assert_ok!(Perun::conclude(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs
		));
		event_concluded(state.channel_id);
	});
}

#[test]
fn conclude_not_final() {
	run_test(|setup| {
		let sigs = sign_state(&setup.state, &setup);

		assert_noop!(
			Perun::conclude(
				Origin::signed(setup.ids.alice),
				setup.params.clone(),
				setup.state.clone(),
				sigs
			),
			pallet_perun::Error::<Test>::StateNotFinal
		);

		assert_no_events();
	});
}

#[test]
fn conclude_invalid_part_num() {
	run_test(|setup| {
		let mut state = setup.state.clone();
		state.finalized = true;
		let bad_sigs = vec![
			vec![], // No parts
		];
		for bad_sig in bad_sigs {
			assert_noop!(
				Perun::conclude(
					Origin::signed(setup.ids.carl),
					setup.params.clone(),
					state.clone(),
					bad_sig
				),
				pallet_perun::Error::<Test>::InvalidParticipantNum
			);
		}
		assert_no_events();
	});
}

#[test]
fn conclude_invalid_sig_nums() {
	run_test(|setup| {
		let mut state = setup.state.clone();
		state.finalized = true;
		let sigs = sign_state(&setup.state, &setup);
		let bad_sigs = vec![
			vec![sigs[0].clone()],                                   // One sig
			vec![sigs[0].clone(), sigs[0].clone(), sigs[0].clone()], // Three sigs
		];
		for bad_sig in bad_sigs {
			assert_noop!(
				Perun::conclude(
					Origin::signed(setup.ids.carl),
					setup.params.clone(),
					state.clone(),
					bad_sig
				),
				pallet_perun::Error::<Test>::InvalidSignatureNum
			);
		}
		assert_no_events();
	});
}

#[test]
fn conclude_invalid_sig() {
	run_test(|setup| {
		let mut state = setup.state.clone();
		let sigs_bad = sign_state(&state, &setup);
		state.finalized = true;
		state.version += 1;
		let sigs_good = sign_state(&state, &setup);
		let sigs = vec![
			vec![sigs_bad[0].clone(), sigs_good[1].clone()], // Alice wrong
			vec![sigs_good[0].clone(), sigs_bad[1].clone()], // Bob wrong
			sigs_bad,                                        // Both wrong
		];

		for sig in sigs {
			assert_noop!(
				Perun::conclude(
					Origin::signed(setup.ids.carl),
					setup.params.clone(),
					state.clone(),
					sig,
				),
				pallet_perun::Error::<Test>::InvalidSignature
			);
		}
		assert_no_events();
	});
}

#[test]
fn conclude_invalid_channel_id() {
	run_test(|setup| {
		let mut state = setup.state.clone();
		state.finalized = true;
		let sigs = sign_state(&state, &setup);
		let mut params = setup.params.clone();
		params.nonce = NonceOf::<Test>::default();

		// Different nonce
		assert_noop!(
			Perun::conclude(Origin::signed(setup.ids.carl), params, state.clone(), sigs),
			pallet_perun::Error::<Test>::InvalidChannelId
		);
		assert_no_events();
	});
}

#[test]
fn conclude_dispute() {
	run_test(|setup| {
		deposit_both(&setup);
		call_dispute(&setup, false);
		let mut state = setup.state.clone();
		state.finalized = true;
		let sigs = sign_state(&state, &setup);

		assert_ok!(Perun::conclude(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs
		));
		event_concluded(state.channel_id);
	});
}

#[test]
/// The participants try to withdraw more funds than they deposited.
fn conclude_insufficient_deposits() {
	run_test(|setup| {
		let mut state = setup.state.clone();
		state.finalized = true;
		// Alice and Bob deposit.
		deposit_both(&setup);

		// Alice will try to withdraw 1 too much.
		state.balances[0] += 1;
		let sigs = sign_state(&state, &setup);

		assert_noop!(
			Perun::conclude(
				Origin::signed(setup.ids.alice),
				setup.params.clone(),
				state,
				sigs
			),
			pallet_perun::Error::<Test>::InsufficientDeposits
		);
	});
}

#[test]
fn conclude_already_concluded_different_version() {
	run_test(|setup| {
		deposit_both(&setup);
		call_dispute(&setup, false);
		let mut state = setup.state.clone();
		state.finalized = true;
		let sigs = sign_state(&state, &setup);

		assert_ok!(Perun::conclude(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs.clone()
		));

		// Concluding twice with a different version errors.
		state.version = state.version + 1;
		let sigs = sign_state(&state, &setup);
		assert_noop!(
			Perun::conclude(
				Origin::signed(setup.ids.alice),
				setup.params.clone(),
				state.clone(),
				sigs
			),
			pallet_perun::Error::<Test>::ConcludedWithDifferentVersion
		);
	});
}

#[test]
fn conclude_twice() {
	run_test(|setup| {
		deposit_both(&setup);
		call_dispute(&setup, false);
		let mut state = setup.state.clone();
		state.finalized = true;
		let sigs = sign_state(&state, &setup);

		assert_ok!(Perun::conclude(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs.clone()
		));

		// Twice works, but no new event will be emitted.
		let events = num_events();
		assert_ok!(Perun::conclude(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs
		));
		assert_num_event(events);
	});
}

#[test]
fn conclude_too_early() {
	run_test(|setup| {
		call_dispute(&setup, false);
		let sigs = sign_state(&setup.state, &setup);

		assert_noop!(
			Perun::conclude(
				Origin::signed(setup.ids.alice),
				setup.params.clone(),
				setup.state.clone(),
				sigs
			),
			pallet_perun::Error::<Test>::ConcludedTooEarly
		);
	});
}
