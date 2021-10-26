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

//! Channel types and type defs.

use crate::*;

use codec::{Decode, Encode};
use sp_core::Hasher;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	RuntimeDebug,
};
use sp_std::prelude::*;

// Type alias.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CurrencyOf<T> = <T as Config>::Currency;
pub type WeightInfoOf<T> = <T as Config>::WeightInfo;
pub type VersionOf<T> = <T as pallet::Config>::Version;
pub type NonceOf<T> = <T as pallet::Config>::Nonce;
pub type ChannelIdOf<T> = <T as pallet::Config>::HashValue;
pub type FundingIdOf<T> = <T as pallet::Config>::HashValue;
pub type SecondsOf<T> = <T as pallet::Config>::Seconds;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type HasherOf<T> = <T as pallet::Config>::Hasher;
pub type PkOf<T> = <T as pallet::Config>::PK;
pub type SigOf<T> = <T as pallet::Config>::Signature;

pub type ParamsOf<T> = Params<NonceOf<T>, PkOf<T>, SecondsOf<T>>;
pub type StateOf<T> = State<ChannelIdOf<T>, VersionOf<T>, BalanceOf<T>>;
pub type RegisteredStateOf<T> = RegisteredState<StateOf<T>, SecondsOf<T>>;
pub type WithdrawalOf<T> = Withdrawal<ChannelIdOf<T>, PkOf<T>, AccountIdOf<T>>;
pub type FundingOf<T> = Funding<ChannelIdOf<T>, PkOf<T>>;

#[derive(Encode, Decode, Default, Clone, PartialEq, RuntimeDebug)]
#[codec(dumb_trait_bound)]
/// Fixed parameters of a channel.
///
/// The values are agreed on by all participants before opening a channel.
pub struct Params<Nonce, PK, Seconds> {
	/// Nonce to make these Params unique. Should be picked randomly.
	pub nonce: Nonce,

	/// Vector of all participants of the channel.
	pub participants: Vec<PK>,

	/// Challenge duration of the channel.
	pub challenge_duration: Seconds,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, RuntimeDebug)]
#[codec(dumb_trait_bound)]
/// Off-Chain state of a channel.
pub struct State<ChannelId, Version, Balance> {
	/// Unique channel ID.
	///
	/// It is calculated from the channel's [Params] with [Params::channel_id].
	/// This locks all parameters in place and ensures that a participant
	/// that signed a state also signed the parameters of a channel.
	pub channel_id: ChannelId,

	/// Version of the state.
	///
	/// Higher version values can override states with lower versions.
	/// An honest participant will never sign two states with the same version.
	pub version: Version,

	/// Balance distribution per participants.
	///
	/// Must be the same size as [Params::participants].
	/// The `balances` of a final state describe the 'outcome' of a channel.
	pub balances: Vec<Balance>,

	/// Whether or not this state is final.
	///
	/// Final states define the last state of a channel.
	/// An honest participant will never sign another state after he signed a
	/// final state.
	pub finalized: bool,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, RuntimeDebug)]
#[codec(dumb_trait_bound)]
/// Off-chain [State] that was registered on-chain.
///
/// Is used to track disputes and concluded channels.
pub struct RegisteredState<State, Seconds> {
	/// The registered state.
	pub state: State,

	/// Timeout in case that it is a dispute.
	///
	/// Has no meaning for final states.
	pub timeout: Seconds,

	/// Set iff a channel is concluded.
	///
	/// This means that no other function than [Pallet::withdraw] can be
	/// called on the channel.
	pub concluded: bool,
}

#[derive(Encode, Decode, Default, Copy, Clone, PartialEq, RuntimeDebug)]
#[codec(dumb_trait_bound)]
/// Withdrawal authorization for on-chain funds.
///
/// This is signed by an off-chain participant too authorize
/// on-chain funds withdrawal to a specific on-chain account.
///
/// NOTE: The signature is not part of the struct.
pub struct Withdrawal<ChannelId, PK, AccountId> {
	/// Channel from with to withdraw.
	pub channel_id: ChannelId,

	/// Off-chain participant to debit.
	pub part: PK,

	/// On-Chain Account to credited.
	pub receiver: AccountId,
}

#[derive(Encode, Decode, Default, Copy, Clone, PartialEq, RuntimeDebug)]
#[codec(dumb_trait_bound)]
/// Funding is exclusively used to calculate funding ids via [Funding::id].
pub struct Funding<ChannelId, PK> {
	pub channel: ChannelId,
	pub part: PK,
}

impl<Nonce, PK, Seconds> Params<Nonce, PK, Seconds>
where
	Params<Nonce, PK, Seconds>: Encode,
{
	/// Calculates the Channel ID of the Params.
	pub fn channel_id<T: Hasher>(&self) -> T::Out {
		let encoded = Encode::encode(&self);
		T::hash(&encoded)
	}
}

impl<ChannelId, Version, Balance> State<ChannelId, Version, Balance>
where
	State<ChannelId, Version, Balance>: Encode,
{
	/// Returns whether `sig` is a valid signature for this State and was
	/// created by `PK`.
	pub fn validate_sig<Sig: Verify<Signer = PK>, PK: IdentifyAccount<AccountId = PK>>(
		&self,
		sig: &Sig,
		pk: &PK,
	) -> bool {
		let msg = Encode::encode(&self);
		sig.verify(&*msg, pk)
	}
}

impl<ChannelId, Pk, AccountId> Withdrawal<ChannelId, Pk, AccountId>
where
	Withdrawal<ChannelId, Pk, AccountId>: Encode,
	Pk: IdentifyAccount<AccountId = Pk>,
{
	/// Returns whether `sig` is a valid signature for this Withdrawal
	/// and was created by the participant that wants to claim the funds.
	pub fn validate_sig<Sig: Verify<Signer = Pk>>(&self, sig: &Sig) -> bool {
		let msg = Encode::encode(&self);
		sig.verify(&*msg, &self.part)
	}
}

impl<ChannelId, PK> Funding<ChannelId, PK>
where
	Funding<ChannelId, PK>: Encode,
{
	/// Calculates the funding id of a participant in a channel.
	pub fn id<H: Hasher>(&self) -> H::Out {
		let encoded = Encode::encode(&self);
		H::hash(&encoded)
	}
}
