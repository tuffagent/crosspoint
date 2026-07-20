use crate::constants::RATE_SCALE;
use crate::errors::CrossPointError;
use anchor_lang::prelude::*;

/// Converts amount at fixed-point rate (scaled by RATE_SCALE) using a u128
/// intermediate so amount * rate never overflows u64 before the final divide.
pub fn convert_amount(amount: u64, rate: u64) -> Result<u64> {
    require!(rate > 0, CrossPointError::InvalidRate);
    let product = (amount as u128)
        .checked_mul(rate as u128)
        .ok_or(CrossPointError::MathOverflow)?;
    let result = product / (RATE_SCALE as u128);
    u64::try_from(result).map_err(|_| CrossPointError::MathOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_to_one_rate_is_identity() {
        assert_eq!(convert_amount(1_000, RATE_SCALE).unwrap(), 1_000);
    }

    #[test]
    fn half_rate_halves_the_amount() {
        assert_eq!(convert_amount(1_000, RATE_SCALE / 2).unwrap(), 500);
    }

    #[test]
    fn double_rate_doubles_the_amount() {
        assert_eq!(convert_amount(1_000, RATE_SCALE * 2).unwrap(), 2_000);
    }

    #[test]
    fn zero_rate_is_rejected() {
        assert!(convert_amount(1_000, 0).is_err());
    }

    #[test]
    fn large_amount_does_not_overflow() {
        // u64::MAX * RATE_SCALE would overflow u64 but must not overflow u128.
        let result = convert_amount(u64::MAX / 2, RATE_SCALE);
        assert!(result.is_ok());
    }

    #[test]
    fn result_exceeding_u64_is_rejected() {
        assert!(convert_amount(u64::MAX, RATE_SCALE * 2).is_err());
    }

    #[test]
    fn zero_amount_converts_to_zero() {
        assert_eq!(convert_amount(0, RATE_SCALE).unwrap(), 0);
    }

    #[test]
    fn a_small_amount_at_a_low_rate_truncates_to_zero() {
        // 1 point at a 0.0001x rate rounds down to 0, not a fraction; callers that treat a
        // zero result as meaningless (e.g. swap_points) must guard against this themselves.
        assert_eq!(convert_amount(1, RATE_SCALE / 10_000).unwrap(), 0);
    }
}
