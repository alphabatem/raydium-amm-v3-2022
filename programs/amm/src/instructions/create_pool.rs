use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Token2022, Mint, TokenAccount};

use crate::libraries::tick_math;
use crate::states::*;

#[derive(Accounts)]
pub struct CreatePool<'info> {
    /// Address paying to create the pool. Can be anyone
    #[account(mut)]
    pub pool_creator: Signer<'info>,

    /// Which config the pool belongs to.
    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// Initialize an account to store the pool state
    #[account(
    init,
    seeds = [
    POOL_SEED.as_bytes(),
    amm_config.key().as_ref(),
    token_mint_0.key().as_ref(),
    token_mint_1.key().as_ref(),
    ],
    bump,
    payer = pool_creator,
    space = PoolState::LEN
    )]
    pub pool_state: AccountLoader<'info, PoolState>,

    /// Token_0 mint, the key must grater then token_1 mint.
    #[account(
    constraint = token_mint_0.key() < token_mint_1.key()
    )]
    pub token_mint_0: Box<InterfaceAccount<'info, Mint>>,

    /// Token_1 mint
    pub token_mint_1: Box<InterfaceAccount<'info, Mint>>,

    /// Token_0 vault for the pool
    #[account(
    init,
    seeds = [
    POOL_VAULT_SEED.as_bytes(),
    pool_state.key().as_ref(),
    token_mint_0.key().as_ref(),
    ],
    bump,
    payer = pool_creator,
    token::mint = token_mint_0,
    token::authority = pool_state
    )]
    pub token_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Token_1 vault for the pool
    #[account(
    init,
    seeds = [
    POOL_VAULT_SEED.as_bytes(),
    pool_state.key().as_ref(),
    token_mint_1.key().as_ref(),
    ],
    bump,
    payer = pool_creator,
    token::mint = token_mint_1,
    token::authority = pool_state
    )]
    pub token_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Initialize an account to store oracle observations, the account must be created off-chain, constract will initialzied it
    pub observation_state: UncheckedAccount<'info>,

    /// Spl token program
    pub token_program: Program<'info, Token2022>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_pool(ctx: Context<CreatePool>, sqrt_price_x64: u128, open_time: u64) -> Result<()> {
    let mut pool_state = ctx.accounts.pool_state.load_init()?;
    let observation_state_loader = initialize_observation_account(
        ctx.accounts.observation_state.to_account_info(),
        &crate::id(),
    )?;

    let tick = tick_math::get_tick_at_sqrt_price(sqrt_price_x64)?;
    #[cfg(feature = "enable-log")]
    msg!(
        "create pool, init_price: {}, init_tick:{}",
        sqrt_price_x64,
        tick
    );

    let bump = *ctx.bumps.get("pool_state").unwrap();
    pool_state.initialize(
        bump,
        sqrt_price_x64,
        open_time,
        tick,
        ctx.accounts.pool_creator.key(),
        ctx.accounts.token_vault_0.key(),
        ctx.accounts.token_vault_1.key(),
        ctx.accounts.amm_config.as_ref(),
        ctx.accounts.token_mint_0.as_ref(),
        ctx.accounts.token_mint_1.as_ref(),
        &observation_state_loader,
    )?;

    emit!(PoolCreatedEvent {
        token_mint_0: ctx.accounts.token_mint_0.key(),
        token_mint_1: ctx.accounts.token_mint_1.key(),
        tick_spacing: ctx.accounts.amm_config.tick_spacing,
        pool_state: ctx.accounts.pool_state.key(),
        sqrt_price_x64,
        tick,
        token_vault_0: ctx.accounts.token_vault_0.key(),
        token_vault_1: ctx.accounts.token_vault_1.key(),
    });
    Ok(())
}

fn initialize_observation_account<'info>(
    observation_account_info: AccountInfo<'info>,
    program_id: &Pubkey,
) -> Result<AccountLoader<'info, ObservationState>> {
    let observation_loader = AccountLoader::<ObservationState>::try_from_unchecked(
        program_id,
        &observation_account_info,
    )?;
    observation_loader.exit(&crate::id())?;
    Ok(observation_loader)
}
