<center>
<h1> SunRe: BitVM2 For Resinsurance </h1>
</center>

Practical implementation of the BitVM2 protocol - adapted from Nero's Protocol at https://github.com/distributed-lab/nero

# BitVM2 Reinsurance Smart Contract

## Overview

This project demonstrates a novel reinsurance smart contract implementation using BitVM2 on the Bitcoin blockchain. It showcases how complex financial instruments can be created on Bitcoin by leveraging off-chain computations with on-chain verification.

## Key Features

- Bitcoin-based reinsurance contract with yield generation
- Off-chain risk assessment and yield calculations
- On-chain verification of off-chain computations
- Dynamic premium and coverage calculations
- Claim processing and contract state management

## How it Works with BitVM2

This contract utilizes BitVM2's capabilities to perform complex computations off-chain while still maintaining the security and trustlessness of the Bitcoin blockchain. Key aspects include:

1. **Off-chain Computation**: Complex operations like risk assessment, yield calculation, and claim verification are performed off-chain.
2. **On-chain Verification**: Results of off-chain computations are verified on-chain using BitVM2's verification mechanisms.
3. **Large Integer Arithmetic**: Utilizes U256 for precise calculations with large numbers.
4. **State Management**: Contract state is updated based on verified off-chain computations.

## Main Components

1. `ReinsuranceContractWithOffChain`: The main struct representing the reinsurance contract.
2. Off-chain data structures:
   - `OffChainRiskAssessment`
   - `OffChainYieldCalculation`
   - `OffChainClaimVerification`
3. Key operations:
   - Pledge verification
   - Risk assessment verification
   - Yield calculation verification
   - Claim verification
   - State update
   - Termination check

## Setup and Testing

### Prerequisites

- Rust programming environment
- Bitcoin-related crates (as specified in the `use` statements)

### Running the Tests

1. Clone the repository
2. Navigate to the project directory
3. Run the tests using Cargo:

## :file_folder: Contents

The project contains multiple crates:

| Crate | Description |
| --- | --- |
| [bitcoin-splitter](bitcoin-splitter/README.md) | A crate for splitting the Bitcoin script into multiple parts as suggested by the recent [^1]). |
| [bitcoin-winternitz](bitcoin-winternitz) | Winternitz Signature and recovery implementation based on BitVM's [`[signatures]`](https://github.com/BitVM/BitVM/tree/main/src/signatures) package. |
| [bitcoin-utils](bitcoin-utils) | Helper package containing implementation of certain fundamental operations and debugging functions. |
| [bitcoin-testscripts](bitcoin-testscripts) | A collection of test scripts for testing BitVM2 concept. |
| [bitcoin-scriptexec](bitcoin-scriptexec) | A helper crate for executing Bitcoin scripts. Fork of [BitVM package](https://github.com/BitVM/rust-bitcoin-scriptexec). |

## Setting up a Local Bitcoin Node

```shell
docker compose up -d
```

> [!WARNING]
> Sometimes Docker Compose may fail at step of creating the volumes, the most simple solution is, in regards of failure, just trying starting it again several times until it works.

Let us create a temporary alias for `bitcoin-cli` from the container like this:

```shell
alias bitcoin-cli="docker compose exec bitcoind bitcoin-cli"
```

Create a fresh wallet for your user:

```shell
bitcoin-cli createwallet "my"
```

> [!WARNING]
> Do not create more than one wallet, otherwise further steps would require
> a bit of modification.

Generate fresh address and store it to environmental variable:

```shell
export ADDRESS=$(bitcoin-cli getnewaddress "main" "bech32")
```

Then mine 101 blocks to your address:

```shell
bitcoin-cli generatetoaddress 101 $ADDRESS
```

> [!NOTE]
> Rewards for mined locally blocks will go to this address, but, by protocol rules, BTCs are mature only after 100 confirmations, so that's why 101 blocks are mined. You can see other in  `immature` balances fields, after executing next command.
>
> For more info about Bitcoin RPC API see [^2].

```shell
bitcoin-cli getbalances
```

[^1]: https://bitvm.org/bitvm_bridge.pdf
[^2]: https://developer.bitcoin.org/reference/rpc/
