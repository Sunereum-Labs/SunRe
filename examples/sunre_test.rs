use bitcoin_splitter::split::script::{IOPair, SplitableScript};
use bitcoin_utils::treepp::*;
use bitcoin_window_mul::{
    bigint::U256,
    traits::{
        arithmeticable::Arithmeticable,
        integer::{NonNativeInteger, NonNativeLimbInteger},
    },
};

use core::ops::{Add, Mul, Div, Sub};
use num_bigint::BigUint;
use num_traits::One;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

struct OffChainRiskAssessment {
    risk_score: U256,
    assessment_timestamp: U256,
    assessor_signature: [u8; 64],
}

struct OffChainYieldCalculation {
    yield_amount: U256,
    calculation_timestamp: U256,
    calculator_signature: [u8; 64],
}

struct OffChainClaimVerification {
    claim_amount: U256,
    verification_timestamp: U256,
    verifier_signature: [u8; 64],
}

pub struct ReinsuranceContractWithOffChain;

const INPUT_SIZE: usize = 9 * U256::N_LIMBS;
const OUTPUT_SIZE: usize = 3 * U256::N_LIMBS;

const MIN_PLEDGE: u64 = 1_000_000;
const MAX_PLEDGE: u64 = 1_000_000_000;
const BASE_YIELD_RATE: u64 = 500;
const MAX_COVERAGE_MULTIPLIER: u64 = 10;
const PREMIUM_RATE_BP: u64 = 200;

impl SplitableScript<{ INPUT_SIZE }, { OUTPUT_SIZE }> for ReinsuranceContractWithOffChain {
    fn script() -> Script {
        script! {
            { ReinsuranceContractWithOffChain::OP_VERIFY_PLEDGE }
            { ReinsuranceContractWithOffChain::OP_VERIFY_RISK_ASSESSMENT }
            { ReinsuranceContractWithOffChain::OP_VERIFY_YIELD_CALCULATION }
            { ReinsuranceContractWithOffChain::OP_VERIFY_CLAIM_VERIFICATION }
            { ReinsuranceContractWithOffChain::OP_UPDATE_STATE }
            { ReinsuranceContractWithOffChain::OP_CHECK_TERMINATION }
        }
    }

    fn generate_valid_io_pair() -> IOPair<{ INPUT_SIZE }, { OUTPUT_SIZE }> {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let pledged_amount: u64 = prng.gen_range(MIN_PLEDGE..=MAX_PLEDGE);
        let pledged_amount_biguint: BigUint = BigUint::from(pledged_amount);

        let coverage_multiplier: u64 = prng.gen_range(1..=MAX_COVERAGE_MULTIPLIER);
        let coverage_amount: BigUint = pledged_amount_biguint.clone() * coverage_multiplier;

        let premium_amount: BigUint = (&coverage_amount * PREMIUM_RATE_BP) / 10000u64;

        let additional_yield: u64 = (pledged_amount - MIN_PLEDGE) * 200 / (MAX_PLEDGE - MIN_PLEDGE);
        let yield_rate: BigUint = BigUint::from(BASE_YIELD_RATE + additional_yield);

        let time: BigUint = prng.gen_range(BigUint::one()..BigUint::one().shl(32));

        let risk_assessment = OffChainRiskAssessment {
            risk_score: U256::from(prng.gen::<u64>()),
            assessment_timestamp: U256::from(prng.gen::<u64>()),
            assessor_signature: prng.gen::<[u8; 64]>(),
        };

        let yield_amount = (&pledged_amount_biguint * &yield_rate) / 10000u64;
        let yield_calculation = OffChainYieldCalculation {
            yield_amount: U256::from(yield_amount.clone()),
            calculation_timestamp: U256::from(prng.gen::<u64>()),
            calculator_signature: prng.gen::<[u8; 64]>(),
        };

        let claim_verification = OffChainClaimVerification {
            claim_amount: U256::from(prng.gen::<u64>()),
            verification_timestamp: U256::from(prng.gen::<u64>()),
            verifier_signature: prng.gen::<[u8; 64]>(),
        };

        let new_state: BigUint = pledged_amount_biguint.clone() + premium_amount - yield_amount.clone();
        let yield_payout: BigUint = yield_amount;
        let claim_payout: BigUint = claim_verification.claim_amount.into();

        println!("--- Reinsurance Policy Details ---");
        println!("Pledged Amount: {} satoshis", pledged_amount);
        println!("Coverage Amount: {} satoshis", coverage_amount);
        println!("Premium Amount: {} satoshis", premium_amount);
        println!("Yield Rate: {}%", yield_rate.clone() / 100u64);
        println!("Yield Payout: {} satoshis", yield_payout);
        println!("-----------------------------------");

        IOPair {
            input: script! {
                { U256::OP_PUSH_U32LESLICE(&pledged_amount_biguint.to_u32_digits()) }
                { U256::OP_PUSH_U32LESLICE(&coverage_amount.to_u32_digits()) }
                { U256::OP_PUSH_U32LESLICE(&premium_amount.to_u32_digits()) }
                { U256::OP_PUSH_U32LESLICE(&yield_rate.to_u32_digits()) }
                { U256::OP_PUSH_U32LESLICE(&time.to_u32_digits()) }
                { U256::OP_PUSH_U32LESLICE(&risk_assessment.risk_score.to_u32_digits()) }
                { U256::OP_PUSH_U32LESLICE(&risk_assessment.assessment_timestamp.to_u32_digits()) }
                { risk_assessment.assessor_signature.to_vec() }
                { U256::OP_PUSH_U32LESLICE(&yield_calculation.yield_amount.to_u32_digits()) }
                { U256::OP_PUSH_U32LESLICE(&yield_calculation.calculation_timestamp.to_u32_digits()) }
                { yield_calculation.calculator_signature.to_vec() }
                { U256::OP_PUSH_U32LESLICE(&claim_verification.claim_amount.to_u32_digits()) }
                { U256::OP_PUSH_U32LESLICE(&claim_verification.verification_timestamp.to_u32_digits()) }
                { claim_verification.verifier_signature.to_vec() }
            },
            output: script! {
                { U256::OP_PUSH_U32LESLICE(&new_state.to_u32_digits()) }
                { U256::OP_PUSH_U32LESLICE(&yield_payout.to_u32_digits()) }
                { U256::OP_PUSH_U32LESLICE(&claim_payout.to_u32_digits()) }
            },
        }
    }

    fn generate_invalid_io_pair() -> IOPair<{ INPUT_SIZE }, { OUTPUT_SIZE }> {
        let mut valid_pair = Self::generate_valid_io_pair();
        let mut prng = ChaCha20Rng::seed_from_u64(0);
        let index_to_modify = prng.gen_range(0..OUTPUT_SIZE);
        valid_pair.output[index_to_modify] ^= 1;
        valid_pair
    }
}

impl ReinsuranceContractWithOffChain {
    fn OP_VERIFY_PLEDGE() -> Script {
        script! { OP_1 }
    }

    fn OP_VERIFY_RISK_ASSESSMENT() -> Script {
        script! {
            { 2 } OP_PICK
            { 1 } OP_PICK
            { 0 } OP_PICK
            OP_3DUP
            OP_CHECKSIG
            OP_VERIFY
            OP_2OVER
            OP_2OVER
            { U256::OP_SUB(4, 0) }
            { 3600 } OP_LESSTHAN
            OP_VERIFY
        }
    }

    fn OP_VERIFY_YIELD_CALCULATION() -> Script {
        script! {
            { 2 } OP_PICK
            { 1 } OP_PICK
            { 0 } OP_PICK
            OP_3DUP
            OP_CHECKSIG
            OP_VERIFY
            OP_2OVER
            OP_2OVER
            { U256::OP_SUB(4, 0) }
            { 3600 } OP_LESSTHAN
            OP_VERIFY
        }
    }

    fn OP_VERIFY_CLAIM_VERIFICATION() -> Script {
        script! {
            { 2 } OP_PICK
            { 1 } OP_PICK
            { 0 } OP_PICK
            OP_3DUP
            OP_CHECKSIG
            OP_VERIFY
            OP_2OVER
            OP_2OVER
            { U256::OP_SUB(4, 0) }
            { 3600 } OP_LESSTHAN
            OP_VERIFY
        }
    }

    fn OP_UPDATE_STATE() -> Script {
        script! {
            { U256::OP_ADD(0, 2) }
            { U256::OP_VERIFY_YIELD_CALCULATION() } OP_SUB
            { U256::OP_VERIFY_CLAIM_VERIFICATION() } OP_SUB
        }
    }

    fn OP_CHECK_TERMINATION() -> Script {
        script! {
            { 4 } OP_PICK
            { 1000 } OP_GREATERTHAN
            OP_IF
                { U256::OP_UPDATE_STATE() }
                { 0 } { 0 }
            OP_ENDIF
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin_splitter::split::core::SplitType;

    #[test]
    fn test_verify() {
        assert!(ReinsuranceContractWithOffChain::verify_random());
    }

    #[test]
    fn test_contract_execution() {
        let IOPair { input, output } = ReinsuranceContractWithOffChain::generate_valid_io_pair();
        
        let split_result = ReinsuranceContractWithOffChain::default_split(input.clone(), SplitType::ByInstructions);
        let last_state = split_result.must_last_state();

        let verification_script = script! {
            { stack_to_script(&last_state.stack) }
            { output }
            { U256::OP_EQUAL(0, 3) }
            { U256::OP_EQUAL(1, 4) }
            { U256::OP_EQUAL(2, 5) }
            OP_BOOLAND
            OP_BOOLAND
        };

        let result = execute_script(verification_script);
        assert!(result.success, "Contract execution verification failed");
    }

    #[test]
    fn test_yield_calculation() {
        let test_pledges = vec![
            MIN_PLEDGE,
            (MIN_PLEDGE + MAX_PLEDGE) / 2,
            MAX_PLEDGE,
        ];

        for pledge in test_pledges {
            let yield_rate = BASE_YIELD_RATE + (pledge - MIN_PLEDGE) * 200 / (MAX_PLEDGE - MIN_PLEDGE);
            let yield_amount = (BigUint::from(pledge) * BigUint::from(yield_rate)) / 10000u64;
            println!("Pledge: {}, Yield Rate: {}, Yield Amount: {}", pledge, yield_rate, yield_amount);
        }
    }
}
