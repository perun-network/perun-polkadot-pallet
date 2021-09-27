<h1 align="center"><br>
    <a href="https://perun.network/"><img src=".assets/go_perun.png" alt="Perun" width="30%"></a>
<br></h1>

<h4 align="center">Perun Polkadot Pallet</h4>

<p align="center">
  <a href="https://www.apache.org/licenses/LICENSE-2.0.txt"><img src="https://img.shields.io/badge/license-Apache%202-blue" alt="License: Apache 2.0"></a>
  <a href="https://github.com/perun-network/perun-polkadot-pallet/actions/workflows/rust.yml"><img src="https://github.com/perun-network/perun-polkadot-pallet/actions/workflows/rust.yml/badge.svg"></a>
</p>

*Perun Polkadot Pallet* provides [go-perun] state channels for all Substrate compatible blockchains.  
Using it in your blockchain means to include it just like any other Substrate Pallet.

## Repo Structure

* `src/`
  * [lib.rs] pallet logic
  * [types.rs] type definitions
* `tests/`
  * `common/`
    * [mock.rs] test configuration
    * [utils.rs] test helpers
  * [unit.rs] pallet unit tests
  * [conclude.rs], [deposit.rs], [dispute.rs], [withdraw.rs] function unit tests
* [Cargo.toml] module info and dependencies

## Protocol

A state channel is opened by depositing funds for it into the pallet by calling *Deposit*.  
The participants of the channel can then do as many off-chain channel updates as they want.  
When all participants come to the conclusion that the channel should be closed, they set the final flag on the channel state, and call *Conclude*.  
All of them can then withdraw the outcome with *Withdraw*, which closes the channel.  

*Dispute* and *ConcludeDispute* are only needed for the dispute case.  
They allow every participant to enforce the last valid state, i.e., the mutually-signed state with the highest version number.  
A dispute is initiated by calling dispute with the latest available state. A registered state can be refuted by calling dispute with a newer state.
All participants can then withdrawn their funds after the dispute was resolved.

### State diagram

```pre
           ┌────────┐                 ┌─────────────┐            ┌─────────────┐
  Deposit  │        │    Conclude     │             │  Withdraw  │             │
──────────►│  OPEN  ├────────────────►│  CONCLUDED  ├───────────►│  WITHDRAWN  │
           │        │                 │             │            │             │
           └───┬────┘                 └─────────────┘            └─────────────┘
               │                             ▲
               │                             │
            Dispute                          │
               │                             │
               │                             │
               ▼                             │
           ┌────────┐                        │
     ┌─────┤        │     Conclude           │
  Dis│pute │DISPUTED├────────────────────────┘
     └────►│        │
           └────────┘
```

### Functions and Types

Functions
- **Deposit(funding_id, amount)** allows a participant to transfer funds into a channel. It is called by each channel participant in order to open the channel.
- **Conclude(params, state, sigs)** collaboratively closes a channel in one step. Only works if all participants signed the state.
- **Dispute(params, state, sigs)** opens a dispute with the passed *state*. Only works if all participants signed the state.
- **ConcludeDispute(params, channel_id)** concludes a dispute after its timeout ran out.
- **Withdraw(withdrawal, sig)** withdraws the outcome of a channel of a single participants. All participants can call this function after the channel is concluded.

Types
- **Params** defines the constant configuration of a channel.
- **State** represents an off-chain state of a channel.
- **Withdrawal** authorizes an on-chain funds withdrawal.
- **RegisteredState** stores a dispute.
- **Channel ID** (aka *channel_id*) uniquely identifies a channel. Calculated as `Hash(params)`.
- **Funding ID** (aka *funding_id*) uniquely identifies a participant in a channel. Calculated as `Hash(channel_id|participant)`.

### Tests

The tests can be run with:
```bash
cargo test --all-features
```

or in docker:

```bash
# Run this each time you change something.
docker build -t perun .
docker run --rm perun
```

### Documentation

The in-code documentation can be opened with:

```bash
cargo doc --no-deps --open --package pallet-perun
```

## Funding

The development of this project is supported by the [Web3 Foundation] through the [Open Grants Program].  
The development of the go-perun library is supported by the German Ministry of Education and Science (BMBF) through a Startup Secure grant.

## Open issues

Some points that we want to take a closer look at in the future:

- Add codecov once the repo is public.
- Only one currency can currently be used per channel. Multi-Currency channels are supported by *go-perun*. Having them here would be nice too.
- Find out if it is dangerous to have state-modifying functions in not-`#[pallet::call]` functions.
- Add reasonable weight estimations.
- It is possible to dispute unfunded channels. This could be used to inflate the on-chain state.

## Security Disclaimer

This software is still under development.
The authors take no responsibility for any loss of digital assets or other damage caused by the use of it.

## Copyright

Copyright 2021 PolyCrypt GmbH.  
Use of the source code is governed by the Apache 2.0 license that can be found in the [LICENSE file](LICENSE).

<!--- Links -->

[go-perun]: https://github.com/hyperledger-labs/go-perun
[Perun Pallet]: https://github.com/perun-network/perun-polkadot-pallet
[frontend template]: https://github.com/substrate-developer-hub/substrate-front-end-template

[Open Grant]: https://github.com/perun-network/Open-Grants-Program/blob/master/applications/perun_channels.md#w3f-open-grant-proposal
[Web3 Foundation]: https://web3.foundation/about/
[Open Grants Program]: https://github.com/w3f/Open-Grants-Program#open-grants-program-

[lib.rs]: src/lib.rs
[types.rs]: src/types.rs
[utils.rs]: tests/common/utils.rs
[mock.rs]: tests/common/mock.rs
[unit.rs]: tests/unit.rs
[conclude.rs]: tests/conclude.rs
[deposit.rs]: tests/deposit.rs
[dispute.rs]: tests/dispute.rs
[withdraw.rs]: tests/withdraw.rs
[Cargo.toml]: Cargo.toml
