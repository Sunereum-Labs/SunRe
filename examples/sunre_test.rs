// sunre_test.rs

use bitcoin_splitter::split::script::{IOPair, SplitableScript};
use bitcoin_utils::treepp::*;
use bitcoin_window_mul::{
    bigint::U256,
    traits::{
        arithmeticable::Arithmeticable,
        integer::{NonNativeInteger, NonNativeLimbInteger},
    },
};

use bitcoin::hashes::{hash160, Hash};
use bitcoin::util::address::Address;
use bitcoin::network::constants::Network;
use bitcoin::blockdata::script::Script as BitcoinScript;

use core::ops::{Add, Mul, Div, Sub};
use num_bigint::BigUint;
use num_traits::One;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Struct representing the off-chain parametric trigger data.
struct OffChainParametricTrigger {
    trigger_occurred: bool,
    trigger_timestamp: U256,
    oracle_signature: [u8; 64],
}

/// Main struct representing the reinsurance contract with parametric trigger handling.
pub struct ReinsuranceContractWithParametricTrigger;

/// Constants defining the input and output sizes for the script.
const INPUT_SIZE: usize = 9 * U256::N_LIMBS;
const OUTPUT_SIZE: usize = 2 * U256::N_LIMBS;

/// Contract parameters.
const MIN_INVESTMENT: u64 = 1_000_000; // Minimum investment amount in satoshis (0.01 BTC).
const YIELD_RATE_BP: u64 = 500;        // Yield rate in basis points (5%).
const YIELD_DIVISOR: u64 = 10_000;     // Divisor for basis points calculation.

impl SplitableScript<{ INPUT_SIZE }, { OUTPUT_SIZE }> for ReinsuranceContractWithParametricTrigger {
    /// Defines the main script that enforces the contract logic.
    fn script() -> Script {
        script! {
            { ReinsuranceContractWithParametricTrigger::OP_VERIFY_INVESTMENT }
            { ReinsuranceContractWithParametricTrigger::OP_VERIFY_TRIGGER }
            { ReinsuranceContractWithParametricTrigger::OP_PROCESS_PAYOUT }
        }
    }

    /// Generates a valid input-output pair for testing the contract.
    fn generate_valid_io_pair() -> IOPair<{ INPUT_SIZE }, { OUTPUT_SIZE }> {
        // Simulating user input for the investment amount (policy amount).
        let investment_amount: u64 = 5_000_000; // Set the investment amount here (e.g., 5,000,000 satoshis).
        let investment_amount_biguint: BigUint = BigUint::from(investment_amount);

        // Simulate whether the trigger occurred.
        let trigger_occurred = true; // Set to true or false for testing.

        // Generate a fixed timestamp for testing.
        let time: BigUint = BigUint::from(1_630_000_000u64); // Fixed timestamp.

        // Create an off-chain parametric trigger.
        let parametric_trigger = OffChainParametricTrigger {
            trigger_occurred,
            trigger_timestamp: U256::from(time.clone()),
            oracle_signature: [0u8; 64], // Placeholder signature.
        };

        // Yield amount if trigger does not occur.
        let yield_amount: BigUint = (&investment_amount_biguint * YIELD_RATE_BP) / YIELD_DIVISOR;

        // Output the policy details.
        println!("--- Reinsurance Policy Details ---");
        println!("Investment Amount: {} satoshis", investment_amount);
        println!("Trigger Occurred: {}", parametric_trigger.trigger_occurred);
        if parametric_trigger.trigger_occurred {
            println!("Payout to Policyholder: {} satoshis", investment_amount);
        } else {
            println!("Yield Payout to Investor: {} satoshis", yield_amount);
        }
        println!("-----------------------------------");

        // Prepare the input script.
        IOPair {
            input: script! {
                // Push the investment amount.
                { U256::OP_PUSH_U32LESLICE(&investment_amount_biguint.to_u32_digits()) }
                // Push the trigger data.
                { if parametric_trigger.trigger_occurred { OP_TRUE } else { OP_FALSE } }
                { U256::OP_PUSH_U32LESLICE(&parametric_trigger.trigger_timestamp.to_u32_digits()) }
                { parametric_trigger.oracle_signature.to_vec() }
            },
            // Prepare the expected output script.
            output: script! {
                // Output depends on whether the trigger occurred.
                { if parametric_trigger.trigger_occurred {
                    // Payout to policyholder.
                    U256::OP_PUSH_U32LESLICE(&investment_amount_biguint.to_u32_digits())
                } else {
                    // Yield payout to investor.
                    U256::OP_PUSH_U32LESLICE(&yield_amount.to_u32_digits())
                }}
                // Indicate the recipient: 0 for policyholder, 1 for investor.
                { if parametric_trigger.trigger_occurred { OP_0 } else { OP_1 } }
            },
        }
    }

    /// Generates a valid input-output pair where the trigger did not occur.
    fn generate_valid_io_pair_no_trigger() -> IOPair<{ INPUT_SIZE }, { OUTPUT_SIZE }> {
        let mut pair = Self::generate_valid_io_pair();
        // Set trigger_occurred to false.
        pair.input[1] = OP_FALSE;
        // Recalculate yield amount.
        let investment_amount_biguint = BigUint::from(5_000_000u64);
        let yield_amount: BigUint = (&investment_amount_biguint * YIELD_RATE_BP) / YIELD_DIVISOR;
        // Update output.
        pair.output = script! {
            { U256::OP_PUSH_U32LESLICE(&yield_amount.to_u32_digits()) }
            { OP_1 } // Indicate payout to investor.
        };
        pair
    }

    /// Generates an invalid input-output pair for testing the contract's error handling.
    fn generate_invalid_io_pair() -> IOPair<{ INPUT_SIZE }, { OUTPUT_SIZE }> {
        let mut valid_pair = Self::generate_valid_io_pair();
        let mut prng = ChaCha20Rng::seed_from_u64(1);
        let index_to_modify = prng.gen_range(0..OUTPUT_SIZE);
        valid_pair.output[index_to_modify] ^= 1; // Introduce an error.
        valid_pair
    }
}

impl ReinsuranceContractWithParametricTrigger {
    /// Verifies the investment amount and ensures it meets the contract requirements.
    fn OP_VERIFY_INVESTMENT() -> Script {
        script! {
            // Verify that the investment amount is at or above the minimum.
            { U256::OP_DUP(0) }
            { U256::OP_PUSH_UINT(MIN_INVESTMENT) }
            OP_GREATERTHANOREQUAL
            OP_VERIFY
        }
    }

    /// Verifies the parametric trigger using the oracle's signature.
    fn OP_VERIFY_TRIGGER() -> Script {
        script! {
            // Simplified signature verification (placeholder).
            OP_TRUE // In practice, use OP_CHECKSIG with the oracle's public key.
            OP_VERIFY
        }
    }

    /// Processes the payout based on whether the trigger occurred.
    fn OP_PROCESS_PAYOUT() -> Script {
        script! {
            // Check if the trigger occurred.
            { OP_DUP(1) } // trigger_occurred.
            OP_IF
                // Trigger occurred: payout to policyholder.
                OP_DROP                          // Remove the trigger flag.
                { U256::OP_DUP(1) }              // investment_amount.
                { OP_0 }                         // Recipient indicator: 0 = policyholder.
            OP_ELSE
                // Trigger did not occur: calculate and payout yield to investor.
                OP_DROP                          // Remove the trigger flag.
                { U256::OP_DUP(1) }              // investment_amount.
                { U256::OP_PUSH_UINT(YIELD_RATE_BP) } // Yield rate in basis points.
                { U256::OP_MUL(0, 1) }           // Multiply: investment_amount * YIELD_RATE_BP.
                { U256::OP_PUSH_UINT(YIELD_DIVISOR) } // Divisor for basis points.
                { U256::OP_DIV(0, 1) }           // Divide to get the yield amount.
                { OP_SWAP }                      // Bring yield amount to the top.
                { OP_1 }                         // Recipient indicator: 1 = investor.
            OP_ENDIF
        }
    }
}

/// Function to generate a P2WSH address from the contract script.
fn generate_contract_address() -> Address {
    // Get the contract script.
    let contract_script = ReinsuranceContractWithParametricTrigger::script();
    let script_bytes = contract_script.to_bytes();

    // Convert to Bitcoin Script.
    let bitcoin_script = BitcoinScript::from(script_bytes);

    // Compute the SHA256 hash of the script.
    let script_hash = hash160::Hash::hash(&script_bytes);

    // Create the P2WSH address.
    let p2wsh_address = Address::p2wsh(&bitcoin_script, Network::Bitcoin);

    p2wsh_address
}

fn main() {
    // Generate the contract address.
    let address = generate_contract_address();
    println!("Send your investment to this address: {}", address);

    // For testing, generate a valid I/O pair.
    let io_pair = ReinsuranceContractWithParametricTrigger::generate_valid_io_pair();

    // Execute the contract and verify the outcome.
    let split_result = ReinsuranceContractWithParametricTrigger::default_split(
        io_pair.input.clone(),
        SplitType::ByInstructions,
    );
    let last_state = split_result.must_last_state();

    let verification_script = script! {
        { stack_to_script(&last_state.stack) }
        { io_pair.output }
        OP_EQUALVERIFY
        OP_TRUE
    };

    let result = execute_script(verification_script);
    if result.success {
        println!("Contract execution verification succeeded.");
    } else {
        println!("Contract execution verification failed.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin_splitter::split::core::SplitType;

    #[test]
    fn test_trigger_occurred() {
        let IOPair { input, output } = ReinsuranceContractWithParametricTrigger::generate_valid_io_pair();

        let split_result = ReinsuranceContractWithParametricTrigger::default_split(input.clone(), SplitType::ByInstructions);
        let last_state = split_result.must_last_state();

        let verification_script = script! {
            { stack_to_script(&last_state.stack) }
            { output }
            OP_EQUALVERIFY
            OP_TRUE
        };

        let result = execute_script(verification_script);
        assert!(result.success, "Contract execution verification failed when trigger occurred");
    }

    #[test]
    fn test_trigger_not_occurred() {
        let IOPair { input, output } = ReinsuranceContractWithParametricTrigger::generate_valid_io_pair_no_trigger();

        let split_result = ReinsuranceContractWithParametricTrigger::default_split(input.clone(), SplitType::ByInstructions);
        let last_state = split_result.must_last_state();

        let verification_script = script! {
            { stack_to_script(&last_state.stack) }
            { output }
            OP_EQUALVERIFY
            OP_TRUE
        };

        let result = execute_script(verification_script);
        assert!(result.success, "Contract execution verification failed when trigger did not occur");
    }
}

