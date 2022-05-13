//! Autogenerated weights for `pallet_perun`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-10-20, STEPS: `1`, REPEAT: 2, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// target/release/node-template
// benchmark
// --execution
// wasm
// --wasm-execution
// compiled
// --chain
// dev
// --pallet
// pallet_perun
// --extrinsic
// *
// --steps
// 1
// --repeat
// 2
// --raw
// --output
// pallets/pallet-perun/src/weights.rs
// --template
// pallets/pallet-perun/template.hbs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

use crate::{types::{ParamsOf, AppRegistry}, Config};

/// Weight functions needed for pallet_perun.
pub trait WeightInfo {
	fn deposit() -> Weight;
	fn dispute(p: u32, ) -> Weight;
	fn progress<T: Config>(params: &ParamsOf<T>) -> Weight;
	fn conclude(p: u32, ) -> Weight;
	fn conclude_final(p: u32, ) -> Weight;
	fn withdraw() -> Weight;
}

/// Weights for pallet_perun using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: PerunModule Deposits (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn deposit() -> Weight {
		(110_609_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: PerunModule StateRegister (r:1 w:1)
	fn dispute(p: u32, ) -> Weight {
		(1_396_000 as Weight)
			// Standard Error: 25_000
			.saturating_add((87_897_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	//TODO: benchmark weight and replace constant
	fn progress<U: Config>(params: &ParamsOf<U>) -> Weight {
		return 10_000 + U::AppRegistry::transition_weight(params);
	}
	// Storage: PerunModule StateRegister (r:1 w:1)
	// Storage: PerunModule Deposits (r:2 w:2)
	fn conclude(p: u32, ) -> Weight {
		(17_600_000 as Weight)
			// Standard Error: 426_000
			.saturating_add((97_182_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(p as Weight)))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
	}
	//TODO: benchmark weight and replace constant
	fn conclude_final(_p: u32, ) -> Weight {
		return 10_000;
	}
	// Storage: PerunModule StateRegister (r:1 w:0)
	// Storage: PerunModule Deposits (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn withdraw() -> Weight {
		(151_546_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: PerunModule Deposits (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn deposit() -> Weight {
		(110_609_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: PerunModule StateRegister (r:1 w:1)
	fn dispute(p: u32, ) -> Weight {
		(1_396_000 as Weight)
			// Standard Error: 25_000
			.saturating_add((87_897_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	//TODO: benchmark weight and replace constant
	fn progress<U: Config>(params: &ParamsOf<U>) -> Weight {
		return 10_000 + U::AppRegistry::transition_weight(params);
	}
	// Storage: PerunModule StateRegister (r:1 w:1)
	// Storage: PerunModule Deposits (r:2 w:2)
	fn conclude(p: u32, ) -> Weight {
		(17_600_000 as Weight)
			// Standard Error: 426_000
			.saturating_add((97_182_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(p as Weight)))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
	}
	//TODO: benchmark weight and replace constant
	fn conclude_final(_p: u32) -> Weight {
		return 10_000;
	}
	// Storage: PerunModule StateRegister (r:1 w:0)
	// Storage: PerunModule Deposits (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn withdraw() -> Weight {
		(151_546_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
}
