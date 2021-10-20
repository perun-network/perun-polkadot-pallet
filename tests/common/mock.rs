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

use pallet_balances;

use super::utils::increment_time;

use frame_support::{parameter_types, PalletId};
use pallet_perun::types::{BalanceOf, FundingIdOf, HasherOf, ParamsOf, StateOf};
use sp_core::{crypto::*, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};
use sp_std::ops::Range;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// For testing the pallet, we construct a mock runtime.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Perun: pallet_perun::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type WeightInfo = ();
}

parameter_types! {
	pub const TimestampMinimumPeriod: u64 = 1;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = TimestampMinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const PerunPalletId: PalletId = PalletId(*b"prnstchs");
	pub const PerunMinDeposit: u64 = 5;
	pub const PerunParticipantNum: Range<u32> = 1..256;
}
impl pallet_perun::Config for Test {
	type Event = Event;
	type PalletId = PerunPalletId;
	type MinDeposit = PerunMinDeposit;
	type ParticipantNum = PerunParticipantNum;
	type Currency = Balances;
	type Version = u32;
	type Nonce = [u8; 32];
	type Signature = sp_core::ecdsa::Signature;
	type PK = sp_core::ecdsa::Public;
	type Hasher = sp_core::KeccakHasher;
	type HashValue = H256;
	type Seconds = u64;
	type WeightInfo = ();
}

pub struct IDs {
	pub alice: u64,
	pub bob: u64,
	pub carl: u64,
	pub dora: u64,
}

pub struct FIDs {
	pub alice: FundingIdOf<Test>,
	pub bob: FundingIdOf<Test>,
}

pub struct KeyPairs {
	pub alice: sp_core::ecdsa::Pair,
	pub bob: sp_core::ecdsa::Pair,
	pub carl: sp_core::ecdsa::Pair,
}

pub struct Setup {
	pub ids: IDs,
	pub keys: KeyPairs,
	pub fids: FIDs,
	pub cid: FundingIdOf<Test>,
	pub state: StateOf<Test>,
	pub params: ParamsOf<Test>,
}

/// Creates a new `Setup` struct.
pub fn new_setup() -> Setup {
	let keys = [
		sp_core::ecdsa::Pair::from_string("//Alice///password", None).unwrap(),
		sp_core::ecdsa::Pair::from_string("//Bob///password2", None).unwrap(),
		sp_core::ecdsa::Pair::from_string("//Carl///password2", None).unwrap(),
	];
	let params = ParamsOf::<Test> {
		nonce: [
			1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1,
			2, 3, 4,
		],
		participants: vec![keys[0].public(), keys[1].public()],
		challenge_duration: 10,
	};
	let cid = params.channel_id::<HasherOf<Test>>();

	Setup {
		ids: IDs {
			alice: 1,
			bob: 2,
			carl: 3,
			dora: 4,
		},
		keys: KeyPairs {
			alice: keys[0].clone(),
			bob: keys[1].clone(),
			carl: keys[2].clone(),
		},
		fids: FIDs {
			alice: Perun::calc_funding_id(cid, &params.participants[0]),
			bob: Perun::calc_funding_id(cid, &params.participants[1]),
		},
		cid: cid,
		state: StateOf::<Test> {
			channel_id: cid,
			version: 123,
			balances: vec![10, 5],
			finalized: false,
		},
		params: params,
	}
}

/// This function builds a genesis block and a setup.
/// The Setup is passed to `test`.
pub fn run_test(test: fn(&Setup) -> ()) {
	let setup = new_setup();
	let mut ext: sp_io::TestExternalities = GenesisConfig {
		// We use default for brevity, but you can configure as desired if needed.
		system: Default::default(),
		balances: pallet_balances::GenesisConfig::<Test> {
			balances: vec![
				(setup.ids.alice, 100),
				(setup.ids.bob, 100),
				(setup.ids.carl, BalanceOf::<Test>::MAX / 2),
				(setup.ids.dora, 1),
			],
		},
	}
	.build_storage()
	.unwrap()
	.into();
	// Start at block 1 to enable event emission.
	ext.execute_with(|| increment_time(1));
	ext.execute_with(|| test(&setup))
}
