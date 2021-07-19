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
use pallet_perun::{types::BalanceOf, Error};

#[test]
/// tests that depositing funds to a funding id works.
fn deposit_some() {
	run_test(|setup| {
		// Holdings are 0.
		assert_eq!(Perun::deposits(setup.fids.alice), None);
		// Alice has 100.
		assert_eq!(Balances::free_balance(setup.ids.alice), 100);
		// Alice deposits 10.
		assert_ok!(Perun::deposit(
			Origin::signed(setup.ids.alice),
			setup.fids.alice,
			10
		));
		// Event emitted.
		event_deposited(setup.fids.alice, 10);
		// Holdings are now 10.
		assert_eq!(Perun::deposits(setup.fids.alice), Some(10));
		// Alice has 90.
		assert_eq!(Balances::free_balance(setup.ids.alice), 90);
	});
}

#[test]
/// Test that the `Deposited` always contains the total deposit
/// and not the relative amount.
fn deposit_event_absolute() {
	run_test(|setup| {
		// Alice deposits 10 and then 20.
		assert_ok!(Perun::deposit(
			Origin::signed(setup.ids.alice),
			setup.fids.alice,
			10
		));
		assert_ok!(Perun::deposit(
			Origin::signed(setup.ids.alice),
			setup.fids.alice,
			20
		));
		// Last event emitted is 30, not 20.
		event_deposited(setup.fids.alice, 30);
		assert_num_event(6);
	});
}

#[test]
/// Tests that depositing less than `MinDeposit` errors and does nothing.
fn deposit_amount_too_low() {
	run_test(|setup| {
		let min = PerunMinDeposit::get();

		// Holdings are now 0.
		assert_eq!(Perun::deposits(setup.fids.alice), None);
		// Carl has `BalanceOf::<Test>::MAX / 2`.
		assert_eq!(
			Balances::free_balance(setup.ids.carl),
			BalanceOf::<Test>::MAX / 2
		);
		// Charlie deposits too few.
		assert_noop!(
			Perun::deposit(Origin::signed(setup.ids.carl), setup.fids.alice, min - 1),
			Error::<Test>::DepositTooSmall
		);
		// Holdings are now 0.
		assert_eq!(Perun::deposits(setup.fids.alice), None);
		// Carl has `BalanceOf::<Test>::MAX / 2`.
		assert_eq!(
			Balances::free_balance(setup.ids.carl),
			BalanceOf::<Test>::MAX / 2
		);
		// No event emitted.
		assert_no_events();
	});
}

#[test]
/// Tests that depositing without having enough balance errors and does nothing.
fn deposit_insufficient_balance() {
	run_test(|setup| {
		// Holdings are 0.
		assert_eq!(Perun::deposits(setup.fids.alice), None);
		// Dora has 1.
		assert_eq!(Balances::free_balance(setup.ids.dora), 1);
		// Dora tries to deposit more than she has.
		assert_noop!(
			Perun::deposit(
				Origin::signed(setup.ids.dora),
				setup.fids.alice,
				PerunMinDeposit::get()
			),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
		// Holdings are 0.
		assert_eq!(Perun::deposits(setup.fids.alice), None);
		// Dora has 1.
		assert_eq!(Balances::free_balance(setup.ids.dora), 1);
		// No event emitted.
		assert_no_events();
	});
}

#[test]
fn deposit_overflow() {
	run_test(|setup| {
		// Alice deposits 10.
		assert_ok!(Perun::deposit(
			Origin::signed(setup.ids.alice),
			setup.fids.alice,
			10
		));

		assert_noop!(
			Perun::deposit(
				Origin::signed(setup.ids.alice),
				setup.fids.alice,
				BalanceOf::<Test>::MAX,
			),
			Error::<Test>::DepositOverflow
		);
	});
}
