use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount};

use crate::error::ErrorCode;

/// Ensures that the signer is the owner or a delgated authority for the position NFT
///
/// # Arguments
///
/// * `signer` - The signer address
/// * `token_account` - The token account holding the position NFT
///
pub fn is_authorized_for_token<'info>(
    signer: &Signer<'info>,
    token_account: &Box<Account<'info, TokenAccount>>,
) -> Result<()> {
    require!(
        token_account.amount == 1 && token_account.owner == signer.key(),
        ErrorCode::NotApproved
    );
    Ok(())
}
