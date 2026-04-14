use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, TransferChecked};
use crate::state::{ProgramConfig, Market, User, UserPosition};
use crate::errors::EscrowError;
use crate::events::{BetPlaced, UserReferrerSet};

/// Place a YES or NO bet.
///
/// - Only callable during the first 15 seconds of a market (betting window).
/// - A user can top-up their existing side by calling again, but CANNOT bet
///   on the opposite side in the same market (use switch_side instead).
/// - On first call: lazily initialises the User PDA (stores referrer) and the
///   UserPosition PDA.  Subsequent calls just update the stake counters.
/// - USDC is immediately transferred from the user's token account to the
///   market vault.
pub fn handler(
    ctx:      Context<PlaceBet>,
    _market_id: u64,  // used only in seeds via #[instruction]; runtime val not needed
    side:     bool,
    amount:   u64,
    referrer: Option<Pubkey>,
) -> Result<()> {
    require!(amount > 0, EscrowError::ZeroAmount);

    let now = Clock::get()?.unix_timestamp;
    require!(ctx.accounts.market.is_betting_open(now), EscrowError::BettingWindowClosed);

    // ── User account (init on first bet) ──────────────────────────────────
    let user_account = &mut ctx.accounts.user_account;
    if user_account.owner == Pubkey::default() {
        user_account.owner = ctx.accounts.user_wallet.key();
        user_account.bump  = ctx.bumps.user_account;

        if let Some(ref_key) = referrer {
            require_keys_neq!(
                ref_key,
                ctx.accounts.user_wallet.key(),
                EscrowError::SelfReferral,
            );
            user_account.referrer = Some(ref_key);
            emit!(UserReferrerSet {
                user:     ctx.accounts.user_wallet.key(),
                referrer: ref_key,
            });
        }
    }
    // Referrer is locked after first set — subsequent `referrer` args are ignored.

    // ── UserPosition (init on first bet for this market) ──────────────────
    let position = &mut ctx.accounts.user_position;
    if position.market == Pubkey::default() {
        position.market      = ctx.accounts.market.key();
        position.user_wallet = ctx.accounts.user_wallet.key();
        position.bump        = ctx.bumps.user_position;
    }

    // ── Enforce single-side rule ──────────────────────────────────────────
    if side {
        require!(position.no_stake == 0, EscrowError::AlreadyOnOtherSide);
        position.yes_stake = position.yes_stake
            .checked_add(amount)
            .ok_or(EscrowError::Overflow)?;
    } else {
        require!(position.yes_stake == 0, EscrowError::AlreadyOnOtherSide);
        position.no_stake = position.no_stake
            .checked_add(amount)
            .ok_or(EscrowError::Overflow)?;
    }

    // ── Update market totals ──────────────────────────────────────────────
    let market = &mut ctx.accounts.market;
    if side {
        market.yes_total = market.yes_total.checked_add(amount).ok_or(EscrowError::Overflow)?;
    } else {
        market.no_total = market.no_total.checked_add(amount).ok_or(EscrowError::Overflow)?;
    }

    let (yes_total, no_total) = (market.yes_total, market.no_total);
    let market_id = market.market_id;

    // ── CPI: USDC from user → vault ───────────────────────────────────────
    let decimals = ctx.accounts.usdc_mint.decimals;
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from:      ctx.accounts.user_token_account.to_account_info(),
                mint:      ctx.accounts.usdc_mint.to_account_info(),
                to:        ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.user_wallet.to_account_info(),
            },
        ),
        amount,
        decimals,
    )?;

    emit!(BetPlaced {
        market_id,
        user:      ctx.accounts.user_wallet.key(),
        side,
        amount,
        yes_total,
        no_total,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct PlaceBet<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, ProgramConfig>,

    #[account(
        mut,
        seeds = [b"market", market_id.to_le_bytes().as_ref()],
        bump  = market.bump,
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump  = market.vault_bump,
        token::mint      = config.usdc_mint,
        token::authority = market,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// User PDA — created on first bet; referrer stored permanently.
    #[account(
        init_if_needed,
        payer = user_wallet,
        space = User::SPACE,
        seeds = [b"user", user_wallet.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, User>,

    /// Per-market position PDA — created on first bet in this market.
    #[account(
        init_if_needed,
        payer = user_wallet,
        space = UserPosition::SPACE,
        seeds = [b"position", market.key().as_ref(), user_wallet.key().as_ref()],
        bump,
    )]
    pub user_position: Account<'info, UserPosition>,

    #[account(mut)]
    pub user_wallet: Signer<'info>,

    #[account(
        mut,
        token::mint      = config.usdc_mint,
        token::authority = user_wallet,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(address = config.usdc_mint)]
    pub usdc_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program:  Program<'info, Token>,
}
