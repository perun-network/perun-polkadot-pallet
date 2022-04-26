//  Copyright 2022 PolyCrypt GmbH
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
use std::convert::TryInto;

use common::mock::*;
use common::utils::*;

use frame_support::assert_noop;
use frame_support::{assert_ok};

#[test]
fn progress() {
	run_test(|setup| {
		deposit_both(&setup);
		call_dispute(&setup, false);

		increment_time(setup.params.challenge_duration);

		let mut state = setup.state.clone();
        state.version += 1;
        state.data = MOCK_DATA_VALID.to_vec();
		let sigs = sign_state(&state, &setup);

        let signer = 0;
        assert_ok!(Perun::progress(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs[signer].clone(),
            signer.try_into().unwrap(),
		));
		event_progressed(state.channel_id);
	});
}

const MOCK_DATA_INVALID: [u8; 1] = [0];

#[test]
fn progress_invalid() {
	run_test(|setup| {
		deposit_both(&setup);
		call_dispute(&setup, false);

		increment_time(setup.params.challenge_duration);

		let mut state = setup.state.clone();
        state.version += 1;
        state.data = MOCK_DATA_INVALID.to_vec();
		let sigs = sign_state(&state, &setup);

        let signer = 0;
        assert_noop!(
            Perun::progress(
                Origin::signed(setup.ids.alice),
                setup.params.clone(),
                state.clone(),
                sigs[signer].clone(),
                signer.try_into().unwrap(),
	    	),
            pallet_perun::Error::<Test>::InvalidTransition
        );
	});
}

#[test]
fn progress_too_early() {
	run_test(|setup| {
		deposit_both(&setup);
		call_dispute(&setup, false);

		increment_time(setup.params.challenge_duration/2);

		let mut state = setup.state.clone();
        state.version += 1;
        state.data = MOCK_DATA_VALID.to_vec();
		let sigs = sign_state(&state, &setup);

        let signer = 0;
        assert_noop!(
            Perun::progress(
                Origin::signed(setup.ids.alice),
                setup.params.clone(),
                state.clone(),
                sigs[signer].clone(),
                signer.try_into().unwrap(),
		    ),
            pallet_perun::Error::<Test>::TooEarly
        );
	});
}

#[test]
fn progress_too_late() {
	run_test(|setup| {
		deposit_both(&setup);
		call_dispute(&setup, false);

		increment_time(setup.params.challenge_duration);

		let mut state = setup.state.clone();
        state.version += 1;
        state.data = MOCK_DATA_VALID.to_vec();
		let sigs = sign_state(&state, &setup);
        let signer = 0;

        assert_ok!(Perun::progress(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs[signer].clone(),
            signer.try_into().unwrap(),
		));
		event_progressed(state.channel_id);

        increment_time(setup.params.challenge_duration);

        state.version += 1;
		let sigs = sign_state(&state, &setup);
        assert_noop!(
            Perun::progress(
                Origin::signed(setup.ids.alice),
                setup.params.clone(),
                state.clone(),
                sigs[signer].clone(),
                signer.try_into().unwrap(),
	    	),
            pallet_perun::Error::<Test>::TooLate
        );
	});
}

#[test]
fn progress_already_concluded() {
	run_test(|setup| {
		deposit_both(&setup);
		call_dispute(&setup, false);

		increment_time(setup.params.challenge_duration);

		let mut state = setup.state.clone();
        state.version += 1;
        state.data = MOCK_DATA_VALID.to_vec();
		let sigs = sign_state(&state, &setup);
        let signer = 0;

        assert_ok!(Perun::progress(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs[signer].clone(),
            signer.try_into().unwrap(),
		));
		event_progressed(state.channel_id);

        increment_time(setup.params.challenge_duration);

		assert_ok!(Perun::conclude(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs
		));
		event_concluded(state.channel_id);

        state.version += 1;
		let sigs = sign_state(&state, &setup);
        assert_noop!(
            Perun::progress(
                Origin::signed(setup.ids.alice),
                setup.params.clone(),
                state.clone(),
                sigs[signer].clone(),
                signer.try_into().unwrap(),
	    	),
            pallet_perun::Error::<Test>::AlreadyConcluded
        );
	});
}