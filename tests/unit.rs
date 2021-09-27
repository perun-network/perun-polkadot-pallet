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

use frame_support::assert_noop;
#[cfg(feature = "expose_privates")]
use pallet_perun::{
	types::{BalanceOf, PkOf},
	Error,
};
use sp_runtime::traits::BadOrigin;

#[cfg(feature = "expose_privates")]
#[test]
fn push_outcome_invalid_parts() {
	run_test(|setup| {
		let parts: Vec<PkOf<Test>> = vec![];
		let bals: Vec<BalanceOf<Test>> = vec![Default::default()];

		assert_noop!(
			Perun::push_outcome_test(setup.cid, &parts, &bals),
			Error::<Test>::InvalidOutcome
		);
	});
}

#[cfg(feature = "expose_privates")]
#[test]
fn push_outcome_invalid_outcome() {
	run_test(|setup| {
		let parts: Vec<PkOf<Test>> = vec![Default::default(); 2];
		let bals: Vec<BalanceOf<Test>> = vec![BalanceOf::<Test>::MAX, 1];

		assert_noop!(
			Perun::push_outcome_test(setup.cid, &parts, &bals),
			Error::<Test>::OutcomeOverflow
		);
	});
}

#[cfg(feature = "expose_privates")]
#[test]
fn push_outcome_insufficient_deposit() {
	run_test(|setup| {
		let parts: Vec<PkOf<Test>> = vec![Default::default(); 1];
		let bals: Vec<BalanceOf<Test>> = vec![1];

		assert_noop!(
			Perun::push_outcome_test(setup.cid, &parts, &bals),
			Error::<Test>::InsufficientDeposits
		);
	});
}

#[test]
fn time_now() {
	run_test(|_| {
		// Time starts at 1 second.
		assert_eq!(1, Perun::now());
		// Advance the time by 10 seconds.
		increment_time(10);
		// Time is now 11 seconds.
		assert_eq!(11, Perun::now());
	});
}

#[test]
/// All functions need signed origins.
fn unsigned_tx() {
	run_test(|_| {
		assert_noop!(
			Perun::deposit(Origin::none(), Default::default(), Default::default()),
			BadOrigin
		);
	});
	run_test(|_| {
		assert_noop!(
			Perun::dispute(
				Origin::none(),
				Default::default(),
				Default::default(),
				Default::default()
			),
			BadOrigin
		);
	});
	run_test(|_| {
		assert_noop!(
			Perun::conclude(
				Origin::none(),
				Default::default(),
				Default::default(),
				Default::default()
			),
			BadOrigin
		);
	});
	run_test(|_| {
		assert_noop!(
			Perun::withdraw(Origin::none(), Default::default(), Default::default()),
			BadOrigin
		);
	});
}
