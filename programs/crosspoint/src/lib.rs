use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod state;
pub mod math;
pub mod token_2022_helpers;

use errors::CrossPointError;

declare_id!("AEJcxEcWkuwo5gu6wPHkJZd4ohbUJ9bR5esFVCQApH4e");

// CrossPoint: cross-merchant loyalty points on Token-2022.
#[program]
pub mod crosspoint {
    use super::*;
}
