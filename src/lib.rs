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

//! *Perun Polkadot Pallet* provides [go-perun](https://github.com/hyperledger-labs/go-perun) state channels for all Substrate compatible blockchains.
//! Using it in your blockchain means to include it just like any other Substrate Pallet.
//! [Perun Polkadot Node](https://github.com/perun-network/perun-polkadot-node) demonstrates this with a minimal approach.

#![cfg_attr(not(feature = "std"), no_std)]
#![doc(html_logo_url = "https://perun.network/images/Asset%2010.svg")]
#![doc(html_favicon_url = "https://perun.network/favicon-32x32.png")]
#![doc(issue_tracker_base_url = "https://github.com/perun-network/perun-polkadot-pallet/issues")]
// Error on broken doc links.
#![deny(rustdoc::broken_intra_doc_links)]

use crate::types::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub use pallet::*;
pub mod weights;

pub mod types;

pub use weights::WeightInfo;

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{Currency, Get, UnixTime},
	PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::traits::{AccountIdConversion, CheckedAdd, IdentifyAccount, Verify};
use sp_std::{cmp, ops::Range, vec::Vec};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::FullCodec;
	use frame_support::{
		dispatch::DispatchResult,
		traits::{ExistenceRequirement, Get},
	};
	use sp_runtime::traits::{CheckedAdd, Member};

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		#[pallet::constant]
		/// ID of this pallet.
		///
		/// Only used to derive the pallets account.
		type PalletId: Get<PalletId>;

		/// Minimal amount that can be deposited to one FundingID.
		///
		/// Use this to prevent the deposits map from being littered.
		#[pallet::constant]
		type MinDeposit: Get<BalanceOf<Self>>;

		/// Valid range for the number of participants in a channel.
		#[pallet::constant]
		type ParticipantNum: Get<Range<u32>>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// On-Chain currency that should be used by the Perun Pallet.
		type Currency: Currency<Self::AccountId>;

		/// Type of a [Params::nonce].
		type Nonce: Encode + Decode + Member;

		/// Type of a [State::version].
		type Version: Encode + Decode + Member + PartialOrd;

		/// Cryptographically secure hashing algorithm that is used to calculate the
		/// ChannelId and FundingId.
		type Hasher: sp_core::Hasher<Out = Self::HashValue>;

		/// Define the output of the Hashing algorithm.
		/// The `FullCodec` ensures that it is usable as a `StorageMap` key.
		type HashValue: FullCodec + Member + Copy;

		/// Off-Chain signature type.
		///
		/// Must be possible to verify that a [Config::PK] created a signature.
		type Signature: Encode + Decode + Member + Verify<Signer = Self::PK>;
		/// PK of a [Config::Signature].
		type PK: Encode + Decode + Member + IdentifyAccount<AccountId = Self::PK>;

		/// Represent a time duration in seconds.
		type Seconds: FullCodec + Member + CheckedAdd + PartialOrd + From<u64>;

		/// Weight info for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn deposits)]
	/// Tracks the deposits as map of FundingId to Balance.
	///
	/// This map can be used to retrieve the balance of each participant in a
	/// channel.
	pub(super) type Deposits<T: Config> =
		StorageMap<_, Blake2_128Concat, FundingIdOf<T>, BalanceOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn state_registers)]
	/// Contains all Disputes and [RegisteredState]s.
	pub(super) type StateRegister<T: Config> =
		StorageMap<_, Blake2_128Concat, ChannelIdOf<T>, RegisteredStateOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	/// All events that can be emitted by Pallet function.
	pub enum Event<T: Config> {
		/// A participant deposited funds into a channel.
		/// \[funding_id, amount\]
		Deposited(FundingIdOf<T>, BalanceOf<T>),

		/// A channel was disputed with the given state.
		/// \[channel_id, state\]
		Disputed(ChannelIdOf<T>, StateOf<T>),

		/// A channel was concluded.
		/// \[channel_od\]
		Concluded(ChannelIdOf<T>),

		/// A participant withdrew funds from a channel.
		/// \[funding_id\]
		Withdrawn(FundingIdOf<T>),
	}

	#[pallet::error]
	/// All errors that can be returned by Pallet functions.
	pub enum Error<T> {
		/// Deposit was less than the configured `MinDeposit`.
		DepositTooSmall,

		/// The dispute timed out and can now be concluded.
		DisputeTimedOut,
		/// There is an ongoing dispute for this channel.
		DisputeActive,
		/// A state cannot be disputed with a state that has a lower version.
		DisputeVersionTooLow,
		/// The challenge duration is too large.
		ChallengeDurationOverflow,

		/// The channel is already concluded.
		AlreadyConcluded,
		/// The dispute timeout did not yet elapse.
		ConcludedTooEarly,
		/// The channel was not concluded.
		NotConcluded,
		// The channel was already concluded but with a different version.
		ConcludedWithDifferentVersion,

		/// The desired outcome overflows the Balance type.
		OutcomeOverflow,
		/// The desired outcome of the channel is invalid.
		InvalidOutcome,
		/// The desired outcome is higher than the initial deposits.
		InsufficientDeposits,
		/// A deposit would overflow the balance type.
		DepositOverflow,

		/// The state was final.
		StateFinal,
		/// The state was not final.
		StateNotFinal,

		/// The passed arguments lead to different Channel IDs.
		InvalidChannelId,
		/// A signature could not be verified.
		InvalidSignature,
		/// Invalid number of signatures.
		/// There must be as many signatures as participants in the params.
		/// Can also be returned if the number of sigs is 0.
		InvalidSignatureNum,
		/// The number of participants did not respect the configured limits.
		InvalidParticipantNum,

		/// The referenced deposit could not be found.
		UnknownDeposit,
		/// The referenced channel could not be found.
		UnknownChannel,
	}

	#[pallet::call]
	/// Contains all user-facing functions.
	impl<T: Config> Pallet<T> {
		/// Deposits funds for a participant into a channel.
		///
		/// The `funding_id` is calculated with [Pallet::calc_funding_id].
		/// The funds are transferred into the pallets custodial Account
		/// from which a participant can withdrawn them when the channel
		/// is concluded with [Pallet::withdraw].
		///
		/// There is no limit on how often or for whom a participant can fund.
		/// The only restriction is that it must be at least [Config::MinDeposit].
		/// Over-funding a channel can result in lost funds.
		///
		/// Emits an [Event::Deposited] event on success.
		#[pallet::weight(WeightInfoOf::<T>::deposit())]
		pub fn deposit(
			origin: OriginFor<T>,
			funding_id: FundingIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(amount >= T::MinDeposit::get(), Error::<T>::DepositTooSmall);
			// Check that a deposit would not overflow, return on failure.
			let holding = <Deposits<T>>::get(&funding_id).unwrap_or_default();
			// An overflow here can happen if a user wants to deposit more than he has.
			let new_holdings = holding
				.checked_add(&amount)
				.ok_or(Error::<T>::DepositOverflow)?;
			// Transfer the funds from the user, return on failure.
			let account_id = Self::account_id();
			T::Currency::transfer(&who, &account_id, amount, ExistenceRequirement::KeepAlive)?;
			// Update the holdings in the deposits map.
			<Deposits<T>>::insert(&funding_id, &new_holdings);
			// Emit the 'Deposited' event.
			Self::deposit_event(Event::Deposited(funding_id, new_holdings));
			Ok(())
		}

		/// Disputes a channel in case of a dishonest participant.
		///
		/// Can only be called with a non-finalized state that is signed by
		/// all participants.
		/// Once a dispute is started, anyone can dispute the channel again
		/// with a state that has a higher [State::version].
		/// A dispute automatically starts a timeout of [Params::challenge_duration]
		/// and can only be re-disputed while it did not run out.
		/// [Pallet::conclude] can be called to conclude the dispute.
		///
		/// Emits an [Event::Disputed] event on success.
		#[pallet::weight(WeightInfoOf::<T>::dispute(
			cmp::min(state_sigs.len() as u32, T::ParticipantNum::get().end)))]
		pub fn dispute(
			origin: OriginFor<T>,
			params: ParamsOf<T>,
			state: StateOf<T>,
			state_sigs: Vec<T::Signature>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			// Final states cannot be disputed.
			ensure!(!state.finalized, Error::<T>::StateFinal);
			Self::validate_fully_signed(&params, &state, state_sigs)?;
			let channel_id = state.channel_id;

			let now = Self::now();
			match <StateRegister<T>>::get(&channel_id) {
				None => {
					let timeout = now
						.checked_add(&params.challenge_duration)
						.ok_or(Error::<T>::ChallengeDurationOverflow)?;
					<StateRegister<T>>::insert(
						channel_id,
						RegisteredState {
							state: state.clone(),
							timeout,
							concluded: false,
						},
					);
					Self::deposit_event(Event::Disputed(channel_id, state));
					Ok(())
				}
				Some(dispute) => {
					ensure!(!dispute.concluded, Error::<T>::AlreadyConcluded);
					// Only register a new dispute iff the timeout still runs
					// a newer version came in.
					ensure!(
						state.version > dispute.state.version,
						Error::<T>::DisputeVersionTooLow
					);
					ensure!(now <= dispute.timeout, Error::<T>::DisputeTimedOut);

					<StateRegister<T>>::insert(
						channel_id,
						RegisteredState {
							state: state.clone(),
							timeout: dispute.timeout,
							concluded: false,
						},
					);
					Self::deposit_event(Event::Disputed(channel_id, state));
					Ok(())
				}
			}
		}
		/// Collaboratively concludes a channel in one step.
		///
		/// This function concludes a channel in the case that all participants
		/// want to close it.
		/// Can only be called with a finalized state that is signed by
		/// all participants.
		///
		/// Emits an [Event::Concluded] event on success.
		#[pallet::weight(WeightInfoOf::<T>::conclude(
			cmp::min(state_sigs.len() as u32, T::ParticipantNum::get().end)))]
		pub fn conclude(
			origin: OriginFor<T>,
			params: ParamsOf<T>,
			state: StateOf<T>,
			state_sigs: Vec<T::Signature>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			Self::validate_fully_signed(&params, &state, state_sigs)?;
			let channel_id = state.channel_id;

			// Check if this channel is being disputed.
			if let Some(dispute) = <StateRegister<T>>::get(&channel_id) {
				if dispute.concluded {
					ensure!(
						dispute.state.version == state.version,
						Error::<T>::ConcludedWithDifferentVersion
					);
					return Ok(());
				}
				// Non-finalized states need to respect the dispute timeout.
				if !state.finalized {
					let now = Self::now();
					ensure!(now > dispute.timeout, Error::<T>::ConcludedTooEarly);
				}
			} else {
				ensure!(state.finalized, Error::<T>::StateNotFinal);
			}

			Self::push_outcome(channel_id, &params.participants, &state.balances)?;
			// Set the channel to `concluded` instead of removing it from the map.
			<StateRegister<T>>::insert(
				channel_id,
				RegisteredState {
					state,
					// Timeout does not matter on finalized disputes.
					timeout: 0.into(),
					concluded: true,
				},
			);
			Self::deposit_event(Event::Concluded(channel_id));
			Ok(())
		}

		/// Withdraws funds from a concluded channel.
		///
		/// Can be called by each participant after a channel was concluded to
		/// withdraw his outcome of the channel.
		/// This is the counterpart to [Pallet::deposit].
		///
		/// Emits an [Event::Withdrawn] event on success.
		#[pallet::weight(WeightInfoOf::<T>::withdraw())]
		pub fn withdraw(
			origin: OriginFor<T>,
			withdrawal: WithdrawalOf<T>,
			withdrawal_sig: SigOf<T>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(
				withdrawal.validate_sig(&withdrawal_sig),
				Error::<T>::InvalidSignature
			);

			match <StateRegister<T>>::get(withdrawal.channel_id) {
				Some(dispute) => {
					ensure!(dispute.concluded, Error::<T>::NotConcluded);
					let funding_id = Self::calc_funding_id(withdrawal.channel_id, &withdrawal.part);
					// Get and remove the deposit.
					match <Deposits<T>>::take(funding_id) {
						Some(deposit) => {
							// Transfer funds.
							let account_id = Self::account_id();
							T::Currency::transfer(
								&account_id,
								&withdrawal.receiver,
								deposit,
								ExistenceRequirement::AllowDeath,
							)?;
							Self::deposit_event(Event::Withdrawn(funding_id));
							Ok(())
						}
						None => Err(Error::<T>::UnknownDeposit.into()),
					}
				}
				None => Err(Error::<T>::UnknownChannel.into()),
			}
		}
	}
}

/// Contains all pallet-facing functions.
impl<T: Config> Pallet<T> {
	/// Returns the account of the pallet.
	/// Cache it if it needed multiple times.
	fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	/// Returns the current time in seconds since
	/// 00:00:00 UTC on 1 January 1970 (unix).
	///
	/// Undefined in the first block.
	pub fn now() -> SecondsOf<T> {
		<pallet_timestamp::Pallet<T> as UnixTime>::now()
			.as_secs()
			.into()
	}

	/// Calculates the funding id of a participant in a channel.
	pub fn calc_funding_id(channel: ChannelIdOf<T>, part: &PkOf<T>) -> FundingIdOf<T> {
		Funding { channel, part }.id::<HasherOf<T>>()
	}

	/// Pushes the outcome of a channel back into the `Deposits` map.
	///
	/// Checks that the sum of withdrawals is smaller or equal to the sum
	/// of deposits per channel.
	/// This ensures that the participants cannot withdraw more than they
	/// initially deposited.
	fn push_outcome(
		channel: ChannelIdOf<T>,
		parts: &[T::PK],
		outcome: &[BalanceOf<T>],
	) -> DispatchResult {
		ensure!(parts.len() == outcome.len(), Error::<T>::InvalidOutcome);
		// Save all Funding IDs for later.
		let mut fids = Vec::<FundingIdOf<T>>::default();
		// Calculate the sums of the outcome and deposit.
		let mut sum_outcome = BalanceOf::<T>::default();
		let mut sum_deposit = BalanceOf::<T>::default();

		for (i, part) in parts.iter().enumerate() {
			let fid = Self::calc_funding_id(channel, part);
			fids.push(fid);
			let deposit = <Deposits<T>>::get(&fid).unwrap_or_default();

			sum_outcome = sum_outcome
				.checked_add(&outcome[i])
				.ok_or(Error::<T>::OutcomeOverflow)?;
			sum_deposit = sum_deposit.checked_add(&deposit).expect(
				"account_id holds the sum of all deposits;\
				The sum of all deposits fits in Balance;\
				Any subsum of deposits fits in Balance;\
				Subsum cannot overflow;\
				qed",
			);
		}
		// Ensure that the participants of a channel can never withdraw more
		// than their initially deposited.
		ensure!(sum_deposit >= sum_outcome, Error::<T>::InsufficientDeposits);
		// Over-funding a channel will result in lost funds.
		// Now we split up all funds according to the outcome.
		for (i, fid) in fids.iter().enumerate() {
			<Deposits<T>>::insert(&fid, outcome[i]);
		}
		Ok(())
	}

	/// Exposes `push_outcome` for testing only.
	#[cfg(feature = "expose_privates")]
	pub fn push_outcome_test(
		channel: ChannelIdOf<T>,
		parts: &[T::PK],
		outcome: &[BalanceOf<T>],
	) -> DispatchResult {
		Self::push_outcome(channel, parts, outcome)
	}

	fn validate_fully_signed(
		params: &ParamsOf<T>,
		state: &StateOf<T>,
		state_sigs: Vec<T::Signature>,
	) -> DispatchResult {
		// The number of participants is valid.
		ensure!(
			T::ParticipantNum::get().contains(&(state_sigs.len() as u32)),
			Error::<T>::InvalidParticipantNum
		);
		// Check that the State and Params match.
		let channel_id = params.channel_id::<T::Hasher>();
		ensure!(state.channel_id == channel_id, Error::<T>::InvalidChannelId);
		// Check the state signatures.
		ensure!(
			state_sigs.len() == params.participants.len(),
			Error::<T>::InvalidSignatureNum
		);
		for (i, sig) in state_sigs.iter().enumerate() {
			ensure!(
				state.validate_sig(sig, &params.participants[i]),
				Error::<T>::InvalidSignature
			);
		}
		Ok(())
	}
}
