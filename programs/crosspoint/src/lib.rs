use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod math;
pub mod state;
pub mod token_2022_helpers;

use instructions::*;

declare_id!("AEJcxEcWkuwo5gu6wPHkJZd4ohbUJ9bR5esFVCQApH4e");

// CrossPoint: cross-merchant loyalty points on Token-2022.
#[program]
pub mod crosspoint {
    use super::*;

    pub fn register_merchant(ctx: Context<RegisterMerchant>, name: String, symbol: String, uri: String) -> Result<()> {
        instructions::register_merchant::handler(ctx, name, symbol, uri)
    }

    pub fn enroll_customer(ctx: Context<EnrollCustomer>) -> Result<()> {
        instructions::enroll_customer::handler(ctx)
    }
}
