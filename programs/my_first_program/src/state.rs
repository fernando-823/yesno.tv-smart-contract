use anchor_lang::prelude::*;

// Phase durations (seconds)
pub const BETTING_WINDOW: i64 = 15;  // 0–15 s  → place bets
pub const SWITCH_WINDOW: i64  = 30;  // 15–30 s → switch side only
pub const DISPUTE_WINDOW: i64 = 180; // 3 minutes after outcome proposed

// ─────────────────────────── ProgramConfig ───────────────────────────────

#[account]
pub struct ProgramConfig {
    pub admin: Pubkey,
    /// USDC SPL mint address (6 decimals on all Solana clusters)
    pub usdc_mint: Pubkey,
    /// Platform treasury token account; receives the platform's 10% share
    pub platform_treasury: Pubkey,
    pub bump: u8,
}

impl ProgramConfig {
    pub const SPACE: usize = 8 + 32 + 32 + 32 + 1; // 105
}

// ─────────────────────────── MarketState ─────────────────────────────────
//
// Borsh serialisation: 1-byte discriminant followed by variant data.
// Largest variant = OutcomeProposed/Disputed = 1 (bool) + 8 (i64) = 9 bytes.
// Total max = 10 bytes.

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum MarketState {
    Open,
    OutcomeProposed { side: bool, proposed_at: i64 },
    Disputed        { side: bool, proposed_at: i64 },
    Finalized       { side: bool },
    Voided,
}

// ─────────────────────────── Market ──────────────────────────────────────

#[account]
pub struct Market {
    pub market_id: u64,
    pub creator:   Pubkey,
    pub open_ts:   i64,
    pub state:     MarketState, // max 10 bytes (see above)

    // Running pool totals — frozen once propose_outcome is called
    pub yes_total: u64,
    pub no_total:  u64,

    // Populated by finalize()
    pub total_pool:         u64,
    pub prize_pool:         u64, // 85 % of total_pool
    pub creator_owed:       u64, //  5 % of total_pool
    pub platform_fee_total: u64, // 10 % of total_pool

    // Incremented by process_split_* ; decremented via claim_platform
    pub platform_accumulated: u64,
    pub platform_claimed:     u64,

    pub creator_fee_claimed: bool,
    pub bump:      u8,
    pub vault_bump: u8,
}

impl Market {
    // 8 discriminator + 8 market_id + 32 creator + 8 open_ts + 10 state
    // + 8*8 numerics + 1 bool + 2 bumps + 64 padding = ~235; use 264.
    pub const SPACE: usize = 8 + 264;

    pub fn is_betting_open(&self, now: i64) -> bool {
        matches!(self.state, MarketState::Open) && now < self.open_ts + BETTING_WINDOW
    }

    pub fn is_switching_open(&self, now: i64) -> bool {
        matches!(self.state, MarketState::Open)
            && now >= self.open_ts + BETTING_WINDOW
            && now  < self.open_ts + SWITCH_WINDOW
    }

    /// Returns the total stake on the winning side (only valid when Finalized).
    pub fn winning_total(&self) -> Option<u64> {
        match &self.state {
            MarketState::Finalized { side } => {
                Some(if *side { self.yes_total } else { self.no_total })
            }
            _ => None,
        }
    }
}

// ─────────────────────────── User ────────────────────────────────────────

#[account]
pub struct User {
    pub owner:    Pubkey,
    /// Referrer pubkey, permanently set on first interaction.
    pub referrer: Option<Pubkey>, // Borsh: 1 byte flag + 32 bytes key = 33 bytes
    pub bump:     u8,
}

impl User {
    pub const SPACE: usize = 8 + 32 + 33 + 1; // 74
}

// ─────────────────────────── UserPosition ────────────────────────────────

#[account]
pub struct UserPosition {
    pub market:      Pubkey,
    pub user_wallet: Pubkey,
    pub yes_stake:   u64,
    pub no_stake:    u64,
    pub claimed_winnings:   bool,
    pub void_claimed:       bool,
    pub platform_split_done: bool,
    pub bump: u8,
}

impl UserPosition {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 8 + 1 + 1 + 1 + 1; // 92

    /// Stake on YES (true) or NO (false).
    pub fn stake_on_side(&self, side: bool) -> u64 {
        if side { self.yes_stake } else { self.no_stake }
    }

    /// Total stake regardless of side (used for void refunds and fee attribution).
    pub fn total_stake(&self) -> u64 {
        self.yes_stake + self.no_stake
    }
}

// ─────────────────────────── AffiliateClaim ──────────────────────────────
//
// One PDA per (market, referrer) pair. Accumulates as process_split_referred
// processes individual positions; referrer claims via claim_affiliate.

#[account]
pub struct AffiliateClaim {
    pub market:     Pubkey,
    pub referrer:   Pubkey,
    pub amount_owed: u64,
    pub claimed:    bool,
    pub bump:       u8,
}

impl AffiliateClaim {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 1 + 1; // 82
}
