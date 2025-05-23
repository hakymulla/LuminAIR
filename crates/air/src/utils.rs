use std::sync::atomic::{AtomicU32, Ordering};

use num_traits::Zero;
use stwo_prover::core::backend::simd::{m31::LOG_N_LANES, qm31::PackedSecureField};

use crate::LuminairInteractionClaim;

/// Calculates the logarithmic size of the trace based on the maximum size of the data.
#[inline]
pub fn calculate_log_size(max_size: usize) -> u32 {
    ((max_size + (1 << LOG_N_LANES) - 1) >> LOG_N_LANES)
        .next_power_of_two()
        .trailing_zeros()
        + LOG_N_LANES
}

/// Verifies the validity of the interaction claim by checking if the sum of claimed sums is zero.
pub fn log_sum_valid(interaction_claim: &LuminairInteractionClaim) -> bool {
    let mut sum = PackedSecureField::zero();

    for claim_opt in [
        &interaction_claim.add,
        &interaction_claim.mul,
        &interaction_claim.sum_reduce,
        &interaction_claim.recip,
        &interaction_claim.max_reduce,
        &interaction_claim.sin,
        &interaction_claim.sin_lookup,
    ] {
        if let Some(ref int_cl) = claim_opt {
            sum += int_cl.claimed_sum.into();
        }
    }

    sum.is_zero()
}

/// Generates a vector of logarithmic sizes for the 'is_first' trace columns.
#[inline]
pub fn get_is_first_log_sizes(max_log_size: u32) -> Vec<u32> {
    let padded_max = max_log_size + 2;
    (4..=padded_max).rev().collect()
}

/// A column of multiplicities for lookup arguments. Allow increasing the multiplicity at a give
/// index. This version uses atomic operations to increase the multiplicity and is `Send`.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AtomicMultiplicityColumn {
    pub data: Vec<AtomicU32>,
}

impl AtomicMultiplicityColumn {
    /// Creates a new `AtomicMultiplicityColumn` with the given size.
    /// The elements are initialized to 0.
    pub fn new(size: u32) -> Self {
        Self {
            data: (0..size).map(|_| AtomicU32::new(0)).collect(),
        }
    }

    #[inline]
    pub fn increase_at(&mut self, address: usize) {
        self.data[address].fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the size of the data vector
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the data vector is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Clone for AtomicMultiplicityColumn {
    fn clone(&self) -> Self {
        let mut new_data = Vec::with_capacity(self.len());

        let values: Vec<u32> = self
            .data
            .iter()
            .map(|atomic| atomic.load(Ordering::Relaxed))
            .collect();

        for val in values {
            new_data.push(AtomicU32::new(val));
        }

        Self { data: new_data }
    }
}
