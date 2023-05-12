use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Token2022, CloseAccount, TransferChecked, Burn};
use anchor_spl::token_interface::{TokenAccount, Mint};
use crate::states::*;

pub fn transfer_from_user_to_pool_vault<'info>(
    signer: &Signer<'info>,
    from: &InterfaceAccount<'info, TokenAccount>,
    to_vault: &InterfaceAccount<'info, TokenAccount>,
    token_program: &Program<'info, Token2022>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }

    token_2022::transfer_checked(CpiContext::new(
        token_program.to_account_info(),
        TransferChecked {
            from: from.to_account_info(),
            mint: from.to_account_info(),
            to: to_vault.to_account_info(),
            authority: signer.to_account_info(),
        },
    ), amount, decimals)
}

pub fn transfer_from_pool_vault_to_user<'info>(
    pool_state_loader: &AccountLoader<'info, PoolState>,
    from_vault: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    token_program: &Program<'info, Token2022>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    let pool_state = pool_state_loader.load()?;
    let pool_state_seeds = [
        POOL_SEED.as_bytes(),
        &pool_state.amm_config.as_ref(),
        &pool_state.token_mint_0.to_bytes() as &[u8],
        &pool_state.token_mint_1.to_bytes() as &[u8],
        &[pool_state.bump],
    ];

    let decimals;
    if pool_state.token_mint_0.eq(&from_vault.mint) {
        decimals = pool_state.mint_decimals_0
    } else {
        decimals = pool_state.mint_decimals_1
    }

    token_2022::transfer_checked(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            TransferChecked {
                from: from_vault.to_account_info(),
                mint: from_vault.to_account_info(),
                to: to.to_account_info(),
                authority: pool_state_loader.to_account_info(),
            },
            &[&pool_state_seeds[..]],
        ),
        amount, decimals)
}

pub fn close_spl_account<'a, 'b, 'c, 'info>(
    owner: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    close_account: &AccountInfo<'info>,
    token_program: &Program<'info, Token2022>,
    signers_seeds: &[&[&[u8]]],
) -> Result<()> {
    token_2022::close_account(CpiContext::new_with_signer(
        token_program.to_account_info(),
        CloseAccount {
            account: close_account.to_account_info(),
            destination: destination.to_account_info(),
            authority: owner.to_account_info(),
        },
        signers_seeds,
    ))
}

pub fn burn<'a, 'b, 'c, 'info>(
    owner: &Signer<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    burn_account: &InterfaceAccount<'info, TokenAccount>,
    token_program: &Program<'info, Token2022>,
    signers_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    token_2022::burn(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Burn {
                mint: mint.to_account_info(),
                from: burn_account.to_account_info(),
                authority: owner.to_account_info(),
            },
            signers_seeds,
        ),
        amount,
    )
}
