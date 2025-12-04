use anchor_lang::prelude::*;

#[error_code]
pub enum LaunchpadError {
    #[msg("Token mint supply must be zero")]
    SupplyNonZero,
    #[msg("Token mint freeze authority must not be set")]
    FreezeAuthoritySet,
    #[msg("Token mint must have 6 decimals")]
    InvalidDecimals,
    #[msg("Token mint authority must be the launch signer")]
    InvalidMintAuthority,
}
