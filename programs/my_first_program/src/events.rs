use anchor_lang::prelude::*;

/// Emitted when a new market is created.
#[event]
pub struct MarketCreated {
    pub market_id: u64,
    pub creator:   Pubkey,
    pub open_ts:   i64,
}

/// Emitted on every bet placed (including top-ups during betting window).
#[event]
pub struct BetPlaced {
    pub market_id: u64,
    pub user:      Pubkey,
    pub side:      bool, // true = YES
    pub amount:    u64,
    pub yes_total: u64,
    pub no_total:  u64,
}

/// Emitted when a user switches their entire position to the other side.
#[event]
pub struct SideSwitched {
    pub market_id: u64,
    pub user:      Pubkey,
    pub from_yes:  bool, // true = was YES, moved to NO
    pub amount:    u64,
}

/// Emitted when admin proposes an outcome; starts 3-minute dispute window.
#[event]
pub struct OutcomeProposed {
    pub market_id:   u64,
    pub side:        bool,
    pub proposed_at: i64,
}

/// Emitted when any user raises a dispute during the 3-minute window.
#[event]
pub struct DisputeRaised {
    pub market_id: u64,
    pub disputer:  Pubkey,
    pub at:        i64,
}

/// Emitted when admin finalises the market and locks the prize/fee math.
#[event]
pub struct MarketFinalized {
    pub market_id:         u64,
    pub winning_side:      bool,
    pub prize_pool:        u64,
    pub creator_owed:      u64,
    pub platform_fee_total: u64,
}

/// Emitted when admin voids the market (full refunds enabled).
#[event]
pub struct MarketVoided {
    pub market_id: u64,
}

/// Emitted when a winning user claims their proportional prize.
#[event]
pub struct WinnerClaimed {
    pub market_id: u64,
    pub user:      Pubkey,
    pub amount:    u64,
}

/// Emitted when a user claims their refund from a voided market.
#[event]
pub struct VoidClaimed {
    pub market_id: u64,
    pub user:      Pubkey,
    pub amount:    u64,
}

/// Emitted when the market creator claims their 5 % fee.
#[event]
pub struct CreatorClaimed {
    pub market_id: u64,
    pub creator:   Pubkey,
    pub amount:    u64,
}

/// Emitted once per UserPosition by process_split_* (platform fee attribution).
#[event]
pub struct PlatformSplitProcessed {
    pub market_id:      u64,
    pub user:           Pubkey,
    pub platform_share: u64,
    pub affiliate_share: u64,
    pub referrer:       Option<Pubkey>,
}

/// Emitted when a referrer claims their accumulated affiliate earnings.
#[event]
pub struct AffiliateClaimedEvent {
    pub market_id: u64,
    pub referrer:  Pubkey,
    pub amount:    u64,
}

/// Emitted when the platform admin withdraws accumulated fees to treasury.
#[event]
pub struct PlatformClaimed {
    pub market_id: u64,
    pub amount:    u64,
    pub treasury:  Pubkey,
}

/// Emitted when a user's referrer is permanently stored on-chain (first bet).
#[event]
pub struct UserReferrerSet {
    pub user:     Pubkey,
    pub referrer: Pubkey,
}
