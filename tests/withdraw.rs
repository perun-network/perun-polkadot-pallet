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

use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use pallet_perun::types::WithdrawalOf;
use sp_core::crypto::Pair;

#[test]
fn withdraw_invalid_sig() {
	run_test(|setup| {
		let withdrawal = WithdrawalOf::<Test> {
			channel_id: setup.cid,
			receiver: setup.ids.alice,
			part: setup.keys.alice.public(),
		};

		assert_noop!(
			Perun::withdraw(
				Origin::signed(setup.ids.alice),
				withdrawal,
				Default::default()
			),
			pallet_perun::Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn withdraw_unknown_channel() {
	run_test(|setup| {
		let withdrawal = WithdrawalOf::<Test> {
			channel_id: setup.cid,
			receiver: setup.ids.alice,
			part: setup.keys.alice.public(),
		};
		let sigs = sign_withdrawal(&withdrawal, setup);

		assert_noop!(
			Perun::withdraw(Origin::signed(setup.ids.alice), withdrawal, sigs[0].clone()),
			pallet_perun::Error::<Test>::UnknownChannel
		);
	});
}

#[test]
/// Test that withdrawing from a not concluded channel fails.
/// It is still possible to dispute unfunded channels…
fn withdraw_not_concluded() {
	run_test(|setup| {
		call_dispute(setup, false);
		let withdrawal = WithdrawalOf::<Test> {
			channel_id: setup.cid,
			receiver: setup.ids.alice,
			part: setup.keys.alice.public(),
		};
		let sigs = sign_withdrawal(&withdrawal, setup);

		assert_noop!(
			Perun::withdraw(Origin::signed(setup.ids.alice), withdrawal, sigs[0].clone()),
			pallet_perun::Error::<Test>::NotConcluded
		);
	});
}

#[test]
fn withdraw_unknown_participant() {
	run_test(|setup| {
		let mut state = setup.state.clone();
		state.finalized = true;
		state.balances = vec![0, 0];
		let sigs = sign_state(&state, &setup);
		assert_ok!(Perun::conclude(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs
		));

		let withdrawal = WithdrawalOf::<Test> {
			channel_id: setup.cid,
			receiver: setup.ids.alice,
			part: setup.keys.carl.public(), //  Carl is not part of the channel
		};
		let raw = Encode::encode(&withdrawal);
		let sig_carl = setup.keys.carl.sign(&raw);

		assert_noop!(
			Perun::withdraw(
				Origin::signed(setup.ids.alice),
				withdrawal,
				sig_carl.clone()
			),
			pallet_perun::Error::<Test>::UnknownDeposit
		);
	});
}

#[test]
fn withdraw_ok() {
	run_test(|setup| {
		// Deposit
		{
			deposit_both(&setup);
			assert_eq!(Balances::free_balance(setup.ids.alice), 90);
			assert_eq!(Balances::free_balance(setup.ids.bob), 95);
		}

		let mut state = setup.state.clone();
		state.finalized = true;
		// Update the balances by swapping them.
		state.balances = vec![state.balances[1], state.balances[0]];
		let sigs = sign_state(&state, &setup);

		assert_ok!(Perun::conclude(
			Origin::signed(setup.ids.alice),
			setup.params.clone(),
			state.clone(),
			sigs
		));

		// Withdraw Alice
		{
			let withdrawal = WithdrawalOf::<Test> {
				channel_id: setup.cid,
				receiver: setup.ids.alice,
				part: setup.keys.alice.public(),
			};
			let sigs = sign_withdrawal(&withdrawal, setup);

			assert_ok!(Perun::withdraw(
				Origin::signed(setup.ids.alice),
				withdrawal.clone(),
				sigs[0].clone()
			),);
			event_withdrawn(setup.fids.alice);

			assert_eq!(Balances::free_balance(setup.ids.alice), 95);
			// Withdrawing twice errors.
			assert_noop!(
				Perun::withdraw(Origin::signed(setup.ids.alice), withdrawal, sigs[0].clone()),
				pallet_perun::Error::<Test>::UnknownDeposit
			);
		}
		// Withdraw Bob
		{
			let withdrawal = WithdrawalOf::<Test> {
				channel_id: setup.cid,
				receiver: setup.ids.bob,
				part: setup.keys.bob.public(),
			};
			let sigs = sign_withdrawal(&withdrawal, setup);

			assert_ok!(Perun::withdraw(
				Origin::signed(setup.ids.bob),
				withdrawal.clone(),
				sigs[1].clone()
			),);
			event_withdrawn(setup.fids.bob);

			assert_eq!(Balances::free_balance(setup.ids.bob), 105);
			// Withdrawing twice errors.
			assert_noop!(
				Perun::withdraw(Origin::signed(setup.ids.bob), withdrawal, sigs[1].clone()),
				pallet_perun::Error::<Test>::UnknownDeposit
			);
		}
	});
}
