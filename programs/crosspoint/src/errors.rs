use anchor_lang::prelude::*;

#[error_code]
pub enum CrossPointError {
    #[msg("Trade lane is not active")]
    LaneNotActive,
    #[msg("Merchants must be passed in sorted pubkey order")]
    MerchantsNotSorted,
    #[msg("A merchant cannot open a lane with itself")]
    SelfLaneNotAllowed,
    #[msg("Trade lane is already active; close it before re-proposing")]
    LaneAlreadyActive,
    #[msg("Exchange rate must be greater than zero")]
    InvalidRate,
    #[msg("Insufficient points balance for this operation")]
    InsufficientPoints,
    #[msg("Only the merchant's own authority may perform this action")]
    UnauthorizedMerchantAuthority,
    #[msg("Only the counterparty merchant's authority may accept or close this lane")]
    UnauthorizedLaneCounterparty,
    #[msg("This achievement badge id is not a recognised badge")]
    UnknownBadge,
    #[msg("This achievement has already been claimed")]
    AchievementAlreadyMinted,
    #[msg("The threshold for this achievement has not been reached yet")]
    AchievementNotEligible,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("The provided customer stats account does not belong to this customer and merchant")]
    InvalidCustomerStats,
}
