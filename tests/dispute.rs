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
use pallet_perun::types::{ChannelIdOf, HasherOf, NonceOf, SecondsOf};

#[test]
fn dispute_ok() {
	run_test(|setup| {
		let sigs = sign_state(&setup.state, &setup);

		assert_ok!(Perun::dispute(
			Origin::signed(setup.ids.carl),
			setup.params.clone(),
			setup.state.clone(),
			sigs
		));
		let channel_id = setup.params.channel_id::<HasherOf<Test>>();
		event_disputed(channel_id, setup.state.clone());
	});
}

#[test]
fn dispute_final() {
	run_test(|setup| {
		let mut state = setup.state.clone();
		state.finalized = true;
		let sigs = sign_state(&state, &setup);

		assert_noop!(
			Perun::dispute(
				Origin::signed(setup.ids.carl),
				setup.params.clone(),
				state,
				sigs
			),
			pallet_perun::Error::<Test>::StateFinal
		);
		assert_no_events();
	});
}

#[test]
fn dispute_already_concluded() {
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
		// Dispute again after conclusion with the non-final state.
		let sigs = sign_state(&setup.state, &setup);
		assert_noop!(
			Perun::dispute(
				Origin::signed(setup.ids.alice),
				setup.params.clone(),
				setup.state.clone(),
				sigs
			),
			pallet_perun::Error::<Test>::AlreadyConcluded
		);
	});
}

#[test]
fn dispute_challenge_duration_overflow() {
	run_test(|setup| {
		let mut params = setup.params.clone();
		let mut state = setup.state.clone();
		params.challenge_duration = SecondsOf::<Test>::MAX;
		state.channel_id = params.channel_id::<HasherOf<Test>>();
		let sigs = sign_state(&state, &setup);

		increment_time(1);
		assert_noop!(
			Perun::dispute(Origin::signed(setup.ids.carl), params, state, sigs),
			pallet_perun::Error::<Test>::ChallengeDurationOverflow
		);
		assert_no_events();
	});
}

#[test]
fn dispute_invalid_part_num() {
	run_test(|setup| {
		let bad_sigs = vec![
			vec![], // No parts
		];
		for bad_sig in bad_sigs {
			assert_noop!(
				Perun::dispute(
					Origin::signed(setup.ids.carl),
					setup.params.clone(),
					setup.state.clone(),
					bad_sig
				),
				pallet_perun::Error::<Test>::InvalidParticipantNum
			);
		}
		assert_no_events();
	});
}

#[test]
fn dispute_invalid_sig_nums() {
	run_test(|setup| {
		let sigs = sign_state(&setup.state, &setup);
		let bad_sigs = vec![
			vec![sigs[0].clone()],                                   // One sig
			vec![sigs[0].clone(), sigs[0].clone(), sigs[0].clone()], // Three sigs
		];
		for bad_sig in bad_sigs {
			assert_noop!(
				Perun::dispute(
					Origin::signed(setup.ids.carl),
					setup.params.clone(),
					setup.state.clone(),
					bad_sig
				),
				pallet_perun::Error::<Test>::InvalidSignatureNum
			);
		}
		assert_no_events();
	});
}

#[test]
fn dispute_invalid_sig() {
	run_test(|setup| {
		let mut state = setup.state.clone();
		let sigs_good = sign_state(&state, &setup);
		state.version += 1;
		let sigs_bad = sign_state(&state, &setup);
		let sigs = vec![
			vec![sigs_bad[0].clone(), sigs_good[1].clone()], // Alice wrong
			vec![sigs_good[0].clone(), sigs_bad[1].clone()], // Bob wrong
			sigs_bad,                                        // Both wrong
		];

		for sig in sigs {
			assert_noop!(
				Perun::dispute(
					Origin::signed(setup.ids.carl),
					setup.params.clone(),
					setup.state.clone(),
					sig,
				),
				pallet_perun::Error::<Test>::InvalidSignature
			);
		}
		assert_no_events();
	});
}

#[test]
fn dispute_invalid_channel_id() {
	run_test(|setup| {
		let sigs = sign_state(&setup.state, &setup);
		let mut params = setup.params.clone();
		params.nonce = NonceOf::<Test>::default();

		// Different nonce
		assert_noop!(
			Perun::dispute(
				Origin::signed(setup.ids.carl),
				params,
				setup.state.clone(),
				sigs,
			),
			pallet_perun::Error::<Test>::InvalidChannelId
		);
		assert_no_events();
	});
	run_test(|setup| {
		let sigs = sign_state(&setup.state, &setup);
		let mut params = setup.params.clone();
		params.participants = vec![];

		// Different parts
		assert_noop!(
			Perun::dispute(
				Origin::signed(setup.ids.carl),
				params,
				setup.state.clone(),
				sigs,
			),
			pallet_perun::Error::<Test>::InvalidChannelId
		);
		assert_no_events();
	});
	run_test(|setup| {
		let sigs = sign_state(&setup.state, &setup);
		let mut params = setup.params.clone();
		params.challenge_duration = setup.params.challenge_duration + 1;

		// Different challenge duration
		assert_noop!(
			Perun::dispute(
				Origin::signed(setup.ids.carl),
				params,
				setup.state.clone(),
				sigs,
			),
			pallet_perun::Error::<Test>::InvalidChannelId
		);
		assert_no_events();
	});
	run_test(|setup| {
		let sigs = sign_state(&setup.state, &setup);
		let mut state = setup.state.clone();
		state.channel_id = ChannelIdOf::<Test>::default();

		// Different Channel ID in State
		assert_noop!(
			Perun::dispute(
				Origin::signed(setup.ids.carl),
				setup.params.clone(),
				state.clone(),
				sigs,
			),
			pallet_perun::Error::<Test>::InvalidChannelId
		);
		assert_no_events();
	});
}

#[test]
fn dispute_same_version() {
	run_test(|setup| {
		// Dispute normal
		call_dispute(&setup, false);

		// Dispute again with same state
		let sigs = sign_state(&setup.state, &setup);
		assert_noop!(
			Perun::dispute(
				Origin::signed(setup.ids.carl),
				setup.params.clone(),
				setup.state.clone(),
				sigs
			),
			pallet_perun::Error::<Test>::DisputeVersionTooLow
		);
		// Only one event in total was emitted.
		assert_num_event(1);
	});
}

#[test]
fn dispute_higher_version() {
	run_test(|setup| {
		// Dispute normal
		call_dispute(&setup, false);

		let mut state = setup.state.clone();
		for _ in 1..10 {
			// increase version
			state.version += 1;
			let sigs = sign_state(&state, &setup);

			assert_ok!(Perun::dispute(
				Origin::signed(setup.ids.carl),
				setup.params.clone(),
				state.clone(),
				sigs
			));
			event_disputed(setup.cid, state.clone());
		}
	});
}

#[test]
fn dispute_timeout() {
	run_test(|setup| {
		// Dispute once
		call_dispute(&setup, false);

		increment_time(setup.params.challenge_duration + 1);
		// Dispute with newer version after timeout
		{
			// increase version
			let mut state = setup.state.clone();
			state.version += 1;
			let sigs = sign_state(&state, &setup);

			assert_noop!(
				Perun::dispute(
					Origin::signed(setup.ids.carl),
					setup.params.clone(),
					state.clone(),
					sigs
				),
				pallet_perun::Error::<Test>::DisputeTimedOut
			);
		}
		// Only one event in total was emitted.
		assert_num_event(1);
	});
}
