use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCodes {
    #[msg("You are not allowed to access this function")]
    Unauthorized,
    #[msg("You have exceeded the maximum allowed")]
    ExceedsLimit,
    #[msg("Token not supported")]
    UnsupportedToken,
    #[msg("Invalid amount")]
    InvalidAmount,
}