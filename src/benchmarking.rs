//! Benchmarking setup for pallet-perun.
#![deny(rustdoc::broken_intra_doc_links)]

use super::{types::*, *};

use codec::Encode;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_io::crypto::{sr25519_generate, sr25519_sign};
use sp_std::vec::Vec;

// Benchmarks all extrinsics and calculates weight estimations.
//
// Does not use the `impl_benchmark_test_suite` macro since there is no associated
// keystore without a node. This prevents the benchmarks from being like:
// `cargo test --features=runtime-benchmarks`.
// The node repo provides the necessary setup:
// https://github.com/perun-network/perun-polkadot-node
benchmarks! {
	where_clause {
		where
			BalanceOf<T>: From<u64>,
			FundingIdOf<T>: From<[u8; 32]>,
			VersionOf<T>: From<u32>,
			NonceOf<T>: From<[u8; 32]>,
			SecondsOf<T>: From<u64>,
			PkOf<T>: From<sp_core::sr25519::Public>,
			SigOf<T>: From<sp_core::sr25519::Signature>,
	}

	deposit {
		let alice = setup_account::<T>();
		let fid: FundingIdOf<T> = [255u8; 32].into();
	}: _(RawOrigin::Signed(alice), fid, 500u32.into())

	dispute {
		let p in 2 .. 255;
		let num_parts = p;

		let (alice, pks, params, state, sigs) = gen_conclude_args::<T>(num_parts, false);
	}: _(RawOrigin::Signed(alice), params, state, sigs)

	conclude {
		let p in 2 .. 255;
		let num_parts = p;

		// Create params and state.
		let (alice, pks, params, state, sigs) = gen_conclude_args::<T>(num_parts, true);
		// Deposit
		let fid = Pallet::<T>::calc_funding_id(state.channel_id, &pks[0].into());
		let origin = RawOrigin::Signed(alice.clone()).into();
		Pallet::<T>::deposit(origin, fid, 500000u64.into())?;
	}: _(RawOrigin::Signed(alice), params, state, sigs)

	withdraw {
		let num_parts = 1;

		// Create params and state.
		let (alice, pks, params, state, sigs) = gen_conclude_args::<T>(num_parts, true);
		// Deposit
		let fid = Pallet::<T>::calc_funding_id(state.channel_id, &pks[0].into());
		let origin = RawOrigin::Signed(alice.clone()).into();
		Pallet::<T>::deposit(origin, fid, 500000u64.into())?;

		// Conclude
		let origin = RawOrigin::Signed(alice.clone()).into();
		Pallet::<T>::conclude(origin, params.clone(), state, sigs)?;

		// Withdraw
		let (withdrawal, sig) = gen_withdraw_args::<T>(alice.clone(), pks[0], &params);
	}: _(RawOrigin::Signed(alice), withdrawal, sig)
}

/// Generates arguments for `Pallet::conclude` and `Pallet::dispute`.
fn gen_conclude_args<T: Config>(
	num_parts: u32,
	is_final: bool,
) -> (
	AccountIdOf<T>,
	Vec<sp_core::sr25519::Public>,
	ParamsOf<T>,
	StateOf<T>,
	Vec<SigOf<T>>,
)
where
	BalanceOf<T>: From<u64>,
	FundingIdOf<T>: From<[u8; 32]>,
	VersionOf<T>: From<u32>,
	NonceOf<T>: From<[u8; 32]>,
	SecondsOf<T>: From<u64>,
	PkOf<T>: From<sp_core::sr25519::Public>,
	SigOf<T>: From<sp_core::sr25519::Signature>,
{
	let alice = setup_account::<T>();
	let pks = gen_pks(num_parts);
	// Generate params and state.
	let params = gen_params::<T>(pks.clone());
	let state = gen_state::<T>(&params, is_final);

	// Sign the state with all participants.
	let data = Encode::encode(&state);
	let sigs: Vec<SigOf<T>> = pks.iter().map(|pk| sign(&data, pk).into()).collect();
	(alice, pks, params, state, sigs)
}

/// Generates arguments for `Pallet::withdraw`.
fn gen_withdraw_args<T: Config>(
	alice_id: AccountIdOf<T>,
	alice_pk: sp_core::sr25519::Public,
	params: &ParamsOf<T>,
) -> (WithdrawalOf<T>, SigOf<T>)
where
	BalanceOf<T>: From<u64>,
	FundingIdOf<T>: From<[u8; 32]>,
	VersionOf<T>: From<u32>,
	NonceOf<T>: From<[u8; 32]>,
	SecondsOf<T>: From<u64>,
	PkOf<T>: From<sp_core::sr25519::Public>,
	SigOf<T>: From<sp_core::sr25519::Signature>,
{
	let withdrawal = WithdrawalOf::<T> {
		channel_id: params.channel_id::<HasherOf<T>>(),
		part: alice_pk.into(),
		receiver: alice_id,
	};
	let data = Encode::encode(&withdrawal);
	let sig = sign(&data, &alice_pk);

	(withdrawal, sig.into())
}

/// Returns a whitelisted and funded Account that can be used to send Extrinsics.
fn setup_account<T: Config>() -> AccountIdOf<T>
where
	BalanceOf<T>: From<u64>,
{
	let alice: AccountIdOf<T> = whitelisted_caller();
	// Fund Alice's account.
	CurrencyOf::<T>::make_free_balance_be(&alice, 100000000000000000u64.into());
	alice
}

/// Generates Params. Uses the passed public keys as participants.
fn gen_params<T: Config>(pks: Vec<sp_core::sr25519::Public>) -> ParamsOf<T>
where
	NonceOf<T>: From<[u8; 32]>,
	PkOf<T>: From<sp_core::sr25519::Public>,
	SecondsOf<T>: From<u64>,
{
	let parts: Vec<PkOf<T>> = pks.into_iter().map(|pk| pk.into()).collect();

	Params {
		nonce: [0u8; 32].into(),
		participants: parts,
		challenge_duration: 0u64.into(),
	}
}

/// Generates a State.
fn gen_state<T: Config>(params: &ParamsOf<T>, is_final: bool) -> StateOf<T>
where
	ChannelIdOf<T>: From<[u8; 32]>,
	BalanceOf<T>: From<u64>,
	VersionOf<T>: From<u32>,
{
	let bals: Vec<BalanceOf<T>> = params.participants.iter().map(|_| 10u64.into()).collect();

	State {
		channel_id: params.channel_id::<HasherOf<T>>(),
		version: 0u32.into(),
		balances: bals,
		finalized: is_final,
	}
}

/// Generates public keys that can be used to sign.
pub fn gen_pks(num_parts: u32) -> Vec<sp_core::sr25519::Public> {
	(0..num_parts).map(|_| gen_pk(None)).collect()
}

/// Creates a public key that can later on be used to sign.
/// The secret key remains in the `keystore` if the node.
pub fn gen_pk(seed: Option<Vec<u8>>) -> sp_core::sr25519::Public {
	sr25519_generate(0.into(), seed)
}

/// Signs the payload on behalf of a public key that was created by
/// `gen_pk`.
pub fn sign(payload: &[u8], pubkey: &sp_core::sr25519::Public) -> sp_core::sr25519::Signature {
	sr25519_sign(0.into(), pubkey, payload).unwrap()
}
