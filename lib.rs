//! # AGORA Protocol v3.6
//! 
//! Universal Basic Income on Solana - Human-First Architecture
//! 
//! ## Overview
//! 
//! AGORA Protocol distributes 100 AGORA tokens daily to every verified human.
//! The system uses biometric identity verification to ensure one person = one account,
//! and implements activity-based fees to encourage using AGORA as currency.
//! 
//! ## Key Features
//! 
//! - **Universal Basic Income**: 100 AGORA/day for every verified human
//! - **Retroactive Claims**: Up to 365 days back-payment on registration
//! - **Biometric Deduplication**: Prevents multiple accounts per person
//! - **Annual Liveness Verification**: Proves user is alive (prevents dead person fraud)
//! - **Activity-Based Fees**: Active users pay less, dormant users pay more
//! - **Merchant Auto-Detection**: Volume-based tier system with fee discounts
//! - **Single Business Model**: One business per human (unlimited off-chain divisions)
//! - **Gas Pool System**: SOL subsidies for new users (sponsor tiers, anti-abuse)
//! 
//! ## Architecture
//! 
//! ```text
//! â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
//! â”‚                 AGORA Protocol                       â”‚
//! â”œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¤
//! â”‚  Identity Layer: Civic Pass + Biometric Hash        â”‚
//! â”‚  Distribution Layer: 100 AGORA/day per human        â”‚
//! â”‚  Fee Layer: Activity-based + Merchant tiers         â”‚
//! â”‚  Governance Layer: DAO with 1 person = 1 vote       â”‚
//! â”‚  Blockchain Layer: Solana (Rust/Anchor)             â”‚
//! â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
//! ```
//! 
//! ## Security Model
//! 
//! - **No Country Trust Scores**: ALL governments treated equally (any could be hostile)
//! - **Biometric Proof-of-Life**: eID biometrics must match LIVE scan
//! - **Individual Accountability**: Only fraudsters punished, no collective penalties
//! - **Rate Limiting**: Prevents transaction spam and gas pool drain attacks
//! 
//! ## Fee Structure
//! 
//! | User Type | Condition | Fee |
//! |-----------|-----------|-----|
//! | Active | TX within 7 days | 0.04% |
//! | Normal | TX within 30 days | 0.05% |
//! | Inactive | TX within 90 days | 0.075% |
//! | Dormant | No TX for 90+ days | 0.10% |
//! | Merchant | Based on tier | 0% - 0.0375% |
//! 
//! ## License
//! 
//! Apache 2.0 - See LICENSE file

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, MintTo, Transfer, Burn}
};
use anchor_lang::solana_program::system_instruction;

declare_id!("AGORA111111111111111111111111111111111111111");

// ============================================================================
// CONSTANTS
// ============================================================================

// Token Distribution
pub const DAILY_AMOUNT: u64 = 100_000_000_000;        // 100 AGORA (9 decimals)
pub const MAX_RETROACTIVE_DAYS: u64 = 365;           // 1 year max retroactive
pub const MAX_ACCUMULATION_DAYS: u64 = 30;           // 30 days max accumulation
pub const SECONDS_PER_DAY: i64 = 86400;

// Transfer Fees
pub const BASE_FEE_RATE: u64 = 5;                    // 0.05% = 5 basis points
pub const FEE_DIVISOR: u64 = 10000;                  // For basis point calculation
pub const BURN_PERCENTAGE: u64 = 50;                 // 50% of fees burned

// Rate Limiting
pub const MIN_TIME_BETWEEN_TX: i64 = 60;             // 60 seconds minimum
pub const DAILY_TX_LIMIT: u8 = 50;                   // Max 50 TX/day base

// Merchant Detection - Simple OR Logic
// Qualify via EITHER unique customers OR monthly volume
// This ensures both retail (many customers) and service (high value) merchants qualify

// Unique customer thresholds (for retail-type merchants)
pub const EMERGING_MIN_CUSTOMERS: u32 = 10;
pub const SMALL_MIN_CUSTOMERS: u32 = 25;
pub const MEDIUM_MIN_CUSTOMERS: u32 = 100;
pub const LARGE_MIN_CUSTOMERS: u32 = 500;
pub const ENTERPRISE_MIN_CUSTOMERS: u32 = 2000;

// Monthly volume thresholds in base units (for service-type merchants)
// 1 AGORA = 1_000_000_000 (9 decimals)
pub const EMERGING_MIN_VOLUME: u128 = 1_000_000_000_000;      // 1,000 AGORA
pub const SMALL_MIN_VOLUME: u128 = 5_000_000_000_000;         // 5,000 AGORA
pub const MEDIUM_MIN_VOLUME: u128 = 25_000_000_000_000;       // 25,000 AGORA
pub const LARGE_MIN_VOLUME: u128 = 100_000_000_000_000;       // 100,000 AGORA
pub const ENTERPRISE_MIN_VOLUME: u128 = 500_000_000_000_000;  // 500,000 AGORA

// Activity-based fee multipliers (in basis points, 100 = 1x)
// Users who transact regularly pay less, dormant users pay more
// This encourages using AGORA as currency, not hoarding
pub const ACTIVE_USER_DAYS: i64 = 7;           // TX within 7 days = active
pub const NORMAL_USER_DAYS: i64 = 30;          // TX within 30 days = normal
pub const INACTIVE_USER_DAYS: i64 = 90;        // TX within 90 days = inactive
                                                // Beyond 90 days = dormant

// Fee multipliers (as percentage of base fee, 100 = 1.0x)
pub const ACTIVE_FEE_MULTIPLIER: u64 = 80;     // 0.8x = 20% discount
pub const NORMAL_FEE_MULTIPLIER: u64 = 100;    // 1.0x = standard
pub const INACTIVE_FEE_MULTIPLIER: u64 = 150;  // 1.5x = 50% premium
pub const DORMANT_FEE_MULTIPLIER: u64 = 200;   // 2.0x = 100% premium

// NOTE: All thresholds are DAO-adjustable
// Real economy will determine actual prices (haircut = 500 or 5000 AGORA?)
// DAO must tune these based on observed transaction patterns

// Security
pub const SUSPENSION_DAYS: i64 = 30;                 // First violation
pub const MERCHANT_DECAY_DAYS: i64 = 90;             // Inactivity before decay
pub const MAX_METADATA_URI_LENGTH: usize = 128;      // IPFS CID or Arweave hash

// ============================================================================
// CHILD PROTECTION CONSTANTS
// ============================================================================
// Philosophy: Protect children's wealth while allowing reasonable access.
// Method: Two pools - accessible and locked. Locked unlocks at 18.
// Mathematical constants used for elegance and consistency.

use std::f64::consts::{PI, E};
pub const PHI: f64 = 1.618033988749895;              // Golden ratio φ

// Accessible rate: portion of each claim child can access immediately
// Under 16: 1/(π×e) ≈ 11.69% accessible, rest locked
// 16-18:    φ/(π×e) ≈ 18.92% accessible, rest locked (maturity boost)
// 18+:      100% accessible + all locked funds unlock

pub const CHILD_ACCESSIBLE_RATE_UNDER_16: f64 = 1.0 / (PI * E);      // ~11.69%
pub const CHILD_ACCESSIBLE_RATE_16_TO_18: f64 = PHI / (PI * E);      // ~18.92%

// Age thresholds in days
pub const CHILD_MATURITY_BOOST_AGE_DAYS: u64 = 5840;   // 16 years (16 × 365)
pub const CHILD_ADULT_AGE_DAYS: u64 = 6570;            // 18 years (18 × 365)

// For integer math (multiply then divide by CHILD_RATE_DIVISOR)
pub const CHILD_RATE_UNDER_16: u64 = 1169;             // 11.69% × 10000
pub const CHILD_RATE_16_TO_18: u64 = 1892;             // 18.92% × 10000
pub const CHILD_RATE_DIVISOR: u64 = 10000;

// ============================================================================
// COUNTRY SANCTIONS CONSTANTS
// ============================================================================
// Philosophy: Governments that commit atrocities should face consequences.
// Method: DAO votes to reduce UBI for citizens of offending countries.
// IMPORTANT: This punishes governments economically, not individuals permanently.
// Citizens are also victims - they still receive reduced UBI, not zero.
// Sanction is temporary - forces governments to change behavior.

pub const DEFAULT_SANCTION_RATE: u64 = 1000;           // 10% of normal UBI (10% × 10000)
pub const DEFAULT_SANCTION_DURATION: i64 = 120 * SECONDS_PER_DAY;  // ~4 months
pub const MIN_SANCTION_RATE: u64 = 500;                // Minimum 5% (never zero - citizens are victims too)
pub const MAX_SANCTION_DURATION: i64 = 365 * SECONDS_PER_DAY;      // Maximum 1 year
pub const SANCTION_RATE_DIVISOR: u64 = 10000;

// ============================================================================
// DAO PROPOSAL CONSTANTS
// ============================================================================
// Philosophy: Anyone can propose, but spam costs money.
// Method: Proposer must post a bond. If proposal reaches quorum, bond is returned.
// If proposal fails to reach quorum or is spam, bond is forfeited to treasury.
// This prevents attack vectors where adversary floods DAO with garbage proposals.

// Bond amounts in AGORA base units (9 decimals)
// Philosophy: Proposals should be collective effort, not individual spam.
// At 100 AGORA/day, these bonds require pooling resources from multiple people.
pub const PROPOSAL_BOND_STANDARD: u64 = 20_000_000_000_000;      // 20,000 AGORA (~200 days UBI or 20 people)
pub const PROPOSAL_BOND_TREASURY: u64 = 50_000_000_000_000;      // 50,000 AGORA (~500 days UBI or 50 people)
pub const PROPOSAL_BOND_CONSTITUTIONAL: u64 = 75_000_000_000_000; // 75,000 AGORA (~750 days UBI or 75 people)
pub const PROPOSAL_BOND_SANCTION: u64 = 100_000_000_000_000;     // 100,000 AGORA (~1000 days UBI or 100 people)

// Voting thresholds
pub const QUORUM_STANDARD: u64 = 1000;                // 1,000 votes minimum for standard
pub const QUORUM_TREASURY: u64 = 5000;                // 5,000 votes for treasury
pub const QUORUM_CONSTITUTIONAL: u64 = 10000;         // 10,000 votes for protocol changes
pub const QUORUM_SANCTION: u64 = 25000;               // 25,000 votes for sanctions

// Voting periods (in seconds)
pub const VOTING_PERIOD_STANDARD: i64 = 3 * SECONDS_PER_DAY;       // 3 days
pub const VOTING_PERIOD_TREASURY: i64 = 7 * SECONDS_PER_DAY;       // 7 days
pub const VOTING_PERIOD_CONSTITUTIONAL: i64 = 14 * SECONDS_PER_DAY; // 14 days
pub const VOTING_PERIOD_SANCTION: i64 = 14 * SECONDS_PER_DAY;      // 14 days

// Approval thresholds (basis points, 10000 = 100%)
pub const APPROVAL_STANDARD: u64 = 5000;              // >50% for standard
pub const APPROVAL_TREASURY: u64 = 5000;              // >50% for treasury
pub const APPROVAL_CONSTITUTIONAL: u64 = 6700;        // >67% supermajority for protocol
pub const APPROVAL_SANCTION: u64 = 7500;              // >75% supermajority for sanctions

// Bond refund threshold - must reach this % of quorum to get bond back
pub const BOND_REFUND_THRESHOLD: u64 = 5000;          // 50% of quorum = bond returned

// ============================================================================
// PROPOSER REPUTATION SYSTEM
// ============================================================================
// Philosophy: Track record matters. Good proposers pay less, spammers get blocked.
// Method: Reputation calculated automatically from voting results. No manual flagging.
// Formula: bond_multiplier = 1 + (abs(reputation) / 2)

// Reputation changes based on voting outcome
pub const REP_PROPOSAL_PASSED: i32 = 2;               // Proposal passed (>50% yes + quorum)
pub const REP_PROPOSAL_REJECTED: i32 = 1;             // Rejected but reached quorum (>50% no)
pub const REP_NO_QUORUM_50: i32 = -1;                 // Failed to reach 50% of quorum
pub const REP_NO_QUORUM_25: i32 = -2;                 // Failed to reach 25% of quorum
pub const REP_NO_QUORUM_10: i32 = -3;                 // Failed to reach 10% of quorum (obvious spam)

// Quorum thresholds for reputation calculation (basis points of required quorum)
pub const QUORUM_THRESHOLD_50: u64 = 5000;            // 50% of quorum
pub const QUORUM_THRESHOLD_25: u64 = 2500;            // 25% of quorum
pub const QUORUM_THRESHOLD_10: u64 = 1000;            // 10% of quorum

// Ban threshold
pub const REP_THRESHOLD_BAN: i32 = -10;               // At -10: cannot create proposals

// ============================================================================
// ANNUAL LIVENESS VERIFICATION CONSTANTS
// ============================================================================
// Philosophy: Don't trust ANY government. Verify the human is ALIVE.
// Method: Biometrics on eID card must match LIVE biometric scan.
// Result: Dead people cannot claim (corpse can't do iris/fingerprint scan)

// Timing
pub const LIVENESS_INTERVAL: i64 = 365 * SECONDS_PER_DAY;    // Must verify every year
pub const LIVENESS_GRACE_PERIOD: i64 = 30 * SECONDS_PER_DAY; // 30 days after expiry to verify
pub const LIVENESS_WARNING_DAYS: i64 = 60;                    // Warn 60 days before expiry

// Biometric Verification
pub const BIOMETRIC_CHALLENGE_TIMEOUT: i64 = 300;    // 5 minutes to complete verification
pub const MAX_VERIFICATION_ATTEMPTS: u8 = 3;         // Max failed attempts per day
pub const VERIFICATION_LOCKOUT: i64 = 24 * 60 * 60;  // 24h lockout after max attempts

// Supported biometric types (from eID cards)
// User must match what's stored on THEIR eID card
pub const BIOMETRIC_FINGERPRINT: u8 = 1;
pub const BIOMETRIC_IRIS: u8 = 2;
pub const BIOMETRIC_FACE: u8 = 3;

// Hash sizes for biometric templates (privacy-preserving)
pub const BIOMETRIC_HASH_SIZE: usize = 32;           // SHA-256 of biometric template

// ============================================================================
// BIOMETRIC DEDUPLICATION CONSTANTS
// ============================================================================
// eIDAS requires EXACTLY both index fingers - prevents using other fingers
// hash(left_index + right_index) = globally unique identifier
// Same person ALWAYS produces same hash, regardless of country

pub const FINGERPRINT_TEMPLATE_SIZE: usize = 512;    // ISO/IEC 19794-2 template size

// ============================================================================
// GAS POOL CONSTANTS
// ============================================================================
// The Gas Pool subsidizes Solana transaction fees for users.
// Sponsors contribute SOL, receive benefits based on tier.
// Pool is PDA-controlled (no human gatekeepers).

// Sponsor tier thresholds (in lamports, 1 SOL = 1_000_000_000 lamports)
pub const BRONZE_THRESHOLD: u64 = 1_000_000_000;           // 1 SOL
pub const SILVER_THRESHOLD: u64 = 10_000_000_000;          // 10 SOL
pub const GOLD_THRESHOLD: u64 = 100_000_000_000;           // 100 SOL
pub const PLATINUM_THRESHOLD: u64 = 1_000_000_000_000;     // 1,000 SOL
pub const DIAMOND_THRESHOLD: u64 = 10_000_000_000_000;     // 10,000 SOL

// Personal allocation percentages (how much sponsor keeps for their own gas)
// Higher tiers donate more to pool, keep less for personal use
pub const BRONZE_PERSONAL_PCT: u64 = 20;     // 20% personal, 80% to pool
pub const SILVER_PERSONAL_PCT: u64 = 15;     // 15% personal, 85% to pool
pub const GOLD_PERSONAL_PCT: u64 = 10;       // 10% personal, 90% to pool
pub const PLATINUM_PERSONAL_PCT: u64 = 5;    // 5% personal, 95% to pool
pub const DIAMOND_PERSONAL_PCT: u64 = 3;     // 3% personal, 97% to pool

// Fee discounts for sponsors (percentage off AGORA transfer fees)
pub const BRONZE_FEE_DISCOUNT: u64 = 25;     // 25% off transfer fees
pub const SILVER_FEE_DISCOUNT: u64 = 50;     // 50% off transfer fees
pub const GOLD_FEE_DISCOUNT: u64 = 75;       // 75% off transfer fees
pub const PLATINUM_FEE_DISCOUNT: u64 = 100;  // 100% off (free transfers)
pub const DIAMOND_FEE_DISCOUNT: u64 = 100;   // 100% off (free transfers)

// Gas subsidy limits for non-contributors
pub const FREE_TIER_DAILY_TX: u8 = 5;              // Max 5 subsidized TX/day
pub const FREE_TIER_COOLDOWN: i64 = 60;            // 60 seconds between subsidized TX
pub const MIN_SUBSIDIZED_AMOUNT: u64 = 100_000_000_000;  // 100 AGORA minimum for subsidy

// Anti-abuse constants
pub const PING_PONG_WINDOW: i64 = 300;             // 5 minute window for ping-pong detection
pub const MAX_SAME_RECIPIENT_PER_HOUR: u8 = 5;     // Max TX to same address per hour
pub const RAPID_TX_THRESHOLD: u8 = 10;             // More than 10 TX in 1 minute = suspicious

// Emergency brake
pub const DAILY_DRAIN_WARNING: u64 = 100_000_000_000;   // 100 SOL/day = warning
pub const DAILY_DRAIN_EMERGENCY: u64 = 500_000_000_000; // 500 SOL/day = emergency pause

// Pool health
pub const MIN_POOL_BALANCE: u64 = 10_000_000_000;  // 10 SOL minimum before warnings

#[program]
pub mod agora_protocol {
    use super::*;

    // ========================================================================
    // INITIALIZATION
    // ========================================================================

    /// Initializes the AGORA Protocol with all required accounts.
    /// 
    /// This function must be called once before any other protocol operations.
    /// It sets up the token mint, treasury, gas pool, and protocol state.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing:
    ///   - `protocol_state`: PDA that stores global protocol configuration
    ///   - `mint`: The AGORA token mint (9 decimals)
    ///   - `treasury`: Account receiving 50% of transaction fees
    ///   - `gas_pool`: Account for SOL gas subsidies
    ///   - `authority`: Initial protocol authority (transfers to DAO later)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` on success
    /// * `Err` if account initialization fails
    /// 
    /// # Events
    /// 
    /// Emits `ProtocolInitialized` with mint, treasury, gas_pool addresses and timestamp.
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// // Called once during deployment
    /// initialize(ctx)?;
    /// // Protocol is now ready to register users
    /// ```
    /// 
    /// # Security
    /// 
    /// - Can only be called once (PDA prevents re-initialization)
    /// - Authority should be transferred to DAO multisig after launch
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;
        
        protocol.authority = ctx.accounts.authority.key();
        protocol.mint = ctx.accounts.mint.key();
        protocol.treasury = ctx.accounts.treasury.key();
        protocol.gas_pool = ctx.accounts.gas_pool.key();
        protocol.total_users = 0;
        protocol.total_merchants = 0;
        protocol.total_businesses = 0;
        protocol.total_minted = 0;
        protocol.total_burned = 0;
        protocol.launch_timestamp = Clock::get()?.unix_timestamp;
        protocol.emergency_paused = false;
        protocol.business_registration_frozen = false;
        protocol.bump = ctx.bumps.protocol_state;
        
        emit!(ProtocolInitialized {
            mint: protocol.mint,
            treasury: protocol.treasury,
            gas_pool: protocol.gas_pool,
            timestamp: protocol.launch_timestamp,
        });
        
        Ok(())
    }

    // ========================================================================
    // USER REGISTRATION - HUMANS ONLY
    // ========================================================================

    /// Registers a new human user in the AGORA Protocol.
    /// 
    /// This is the entry point for all humans to join AGORA. Each person can only
    /// register once - biometric deduplication prevents multiple accounts.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user accounts and protocol state
    /// * `age_in_days` - User's age in days (determines retroactive claim amount)
    /// * `civic_pass_proof` - Cryptographic proof from Civic Pass verification
    /// * `biometric_hash` - SHA-256 hash of both index fingerprints from eID card
    /// 
    /// # Retroactive Claim Calculation
    /// 
    /// | Age | Initial Claim |
    /// |-----|---------------|
    /// | 0-364 days (baby) | age_in_days Ã— 100 AGORA |
    /// | 365+ days (adult) | 36,500 AGORA (max 1 year) |
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Registration successful, retroactive tokens minted
    /// * `Err(BiometricAlreadyRegistered)` - This person already has an account
    /// * `Err(InvalidCivicPass)` - Civic Pass verification failed
    /// 
    /// # Events
    /// 
    /// Emits `UserRegistered` with user pubkey, claim amount, age, and biometric hash.
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// // Register a 25-year-old adult
    /// register_user(ctx, 9125, civic_proof, fingerprint_hash, *b"USA")?;
    /// // User receives 36,500 AGORA (365 days × 100)
    /// 
    /// // Register a 6-month-old baby
    /// register_user(ctx, 180, civic_proof, fingerprint_hash, *b"GRC")?;
    /// // Baby receives 18,000 AGORA (180 days × 100), locked until age 18
    /// ```
    /// 
    /// # Security
    /// 
    /// - **Biometric Deduplication**: PDA seeded by fingerprint hash - same fingerprints
    ///   will cause transaction to fail (Anchor constraint)
    /// - **eIDAS Compliance**: Uses exactly both index fingers as required by EU regulation
    /// - **Privacy**: Only hash stored, not actual biometric data
    /// - **Citizenship**: ISO 3166-1 alpha-3 code for potential DAO sanctions
    pub fn register_user(
        ctx: Context<RegisterUser>,
        age_in_days: u64,
        civic_pass_proof: Vec<u8>,
        biometric_hash: [u8; 32],              // SHA-256(left_index + right_index)
        citizenship: [u8; 3],                   // ISO 3166-1 alpha-3 (e.g., "USA", "GRC")
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;

        // =====================================================================
        // BIOMETRIC DEDUPLICATION CHECK
        // =====================================================================
        // CRITICAL: Same fingerprints = same person = REJECT
        // eIDAS requires EXACTLY both index fingers - cannot use other fingers
        //
        // The biometric_entry PDA will fail to init if hash already exists
        // This is enforced by Anchor's init constraint - no code needed!

        let biometric_entry = &mut ctx.accounts.biometric_entry;
        biometric_entry.biometric_hash = biometric_hash;
        biometric_entry.user_account = ctx.accounts.user.key();
        biometric_entry.registered_at = current_time;
        biometric_entry.is_active = true;
        biometric_entry.bump = ctx.bumps.biometric_entry;

        // Update registry stats
        let registry = &mut ctx.accounts.biometric_registry;
        registry.total_registered += 1;
        registry.last_updated = current_time;

        // =====================================================================
        // CIVIC PASS / eIDAS VERIFICATION
        // =====================================================================
        verify_civic_pass(&civic_pass_proof)?;

        // =====================================================================
        // INITIALIZE USER ACCOUNT
        // =====================================================================

        user.owner = ctx.accounts.user.key();
        user.civic_pass = ctx.accounts.civic_pass.key();
        user.is_human = true;
        user.registration_timestamp = current_time;
        user.last_claim_timestamp = current_time;
        user.age_in_days_at_registration = age_in_days;
        user.is_child = age_in_days < CHILD_ADULT_AGE_DAYS; // Under 18
        user.citizenship = citizenship;  // ISO 3166-1 alpha-3

        // Calculate birth timestamp from age
        user.birth_timestamp = current_time - (age_in_days as i64 * SECONDS_PER_DAY);

        // Biometric identity - for deduplication only
        user.biometric_hash = biometric_hash;

        // Merchant fields - volume based
        user.merchant_status = MerchantTier::None;
        user.volume_received = 0;
        user.volume_sent = 0;
        user.tx_count_received = 0;
        user.tx_count_sent = 0;
        user.unique_payers = Vec::with_capacity(1000);
        user.unique_payers_count = 0;
        
        // 30-day rolling volume tracking
        user.monthly_volume_received = 0;
        user.monthly_volume_period_start = current_time;
        user.monthly_unique_payers = Vec::with_capacity(500);
        user.monthly_unique_count = 0;

        // SINGLE BUSINESS MODEL
        user.business_profile = None;

        // Security
        user.violations = 0;
        user.suspended_until = 0;
        user.permanently_blacklisted = false;
        user.bump = ctx.bumps.user_state;
        
        // Mint retroactive claim for HUMAN only (max 1 year for everyone)
        let retroactive_amount = calculate_retroactive_claim(age_in_days);
        
        // =====================================================================
        // CHILD PROTECTION: Split into accessible and locked pools
        // =====================================================================
        let (accessible, locked) = if user.is_child {
            let rate = if age_in_days >= CHILD_MATURITY_BOOST_AGE_DAYS {
                CHILD_RATE_16_TO_18  // φ/(π×e) ≈ 18.92%
            } else {
                CHILD_RATE_UNDER_16  // 1/(π×e) ≈ 11.69%
            };
            let accessible_amount = (retroactive_amount * rate) / CHILD_RATE_DIVISOR;
            let locked_amount = retroactive_amount - accessible_amount;
            (accessible_amount, locked_amount)
        } else {
            (retroactive_amount, 0)  // Adults get 100% accessible
        };
        
        user.accessible_balance = accessible;
        user.locked_balance = locked;
        
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.protocol_state.to_account_info(),
                },
                &[&[b"protocol", &[ctx.accounts.protocol_state.bump]]],
            ),
            retroactive_amount,
        )?;
        
        user.total_claimed = retroactive_amount as u128;

        // Update protocol
        let protocol = &mut ctx.accounts.protocol_state;
        protocol.total_users += 1;
        protocol.total_minted += retroactive_amount as u128;

        emit!(UserRegistered {
            user: ctx.accounts.user.key(),
            retroactive_claim: retroactive_amount,
            age_in_days,
            is_human: true,
            biometric_hash,
            accessible_amount: accessible,
            locked_amount: locked,
            timestamp: user.registration_timestamp,
        });

        Ok(())
    }

    // ========================================================================
    // BUSINESS REGISTRATION - ONE PER HUMAN
    // ========================================================================

    /// Registers a business profile for a verified human.
    /// 
    /// Each human can have exactly ONE business on-chain. This limit prevents
    /// storage bloat attacks while allowing unlimited divisions via off-chain metadata.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user state and protocol state
    /// * `business_info` - Business details including name, category, and metadata URI
    /// 
    /// # Business Info Structure
    /// 
    /// ```ignore
    /// BusinessInfo {
    ///     name: "John's Services",           // On-chain name
    ///     category: BusinessCategory::Services,
    ///     tax_id: Some("ATU12345678"),       // Optional VAT/tax ID
    ///     metadata_uri: "ipfs://Qm...",      // Points to divisions, employees, etc.
    /// }
    /// ```
    /// 
    /// # Off-Chain Metadata (IPFS/Arweave)
    /// 
    /// ```json
    /// {
    ///   "divisions": [
    ///     {"name": "John's Plumbing", "type": "plumbing"},
    ///     {"name": "John's Electric", "type": "electrical"}
    ///   ],
    ///   "employees": [...],
    ///   "locations": [...]
    /// }
    /// ```
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Business registered successfully
    /// * `Err(BusinessAlreadyExists)` - User already has a business
    /// * `Err(MustBeVerifiedHuman)` - Only verified humans can register businesses
    /// * `Err(BusinessRegistrationFrozen)` - Emergency freeze is active
    /// * `Err(MetadataUriTooLong)` - URI exceeds 128 characters
    /// 
    /// # Events
    /// 
    /// Emits `BusinessRegistered` with owner, business name, category, and metadata URI.
    /// 
    /// # Security
    /// 
    /// - **One Business Limit**: Prevents spam attacks (10,000 fake businesses)
    /// - **Human Verification**: Only Civic Pass verified users can register
    /// - **Emergency Freeze**: DAO can halt registrations if needed
    pub fn register_business(
        ctx: Context<RegisterBusiness>,
        business_info: BusinessInfo,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let protocol = &ctx.accounts.protocol_state;
        
        // Check if business registration is frozen (emergency)
        require!(
            !protocol.business_registration_frozen,
            ErrorCode::BusinessRegistrationFrozen
        );
        
        // CRITICAL: Only verified humans can have businesses
        require!(
            user.is_human && user.civic_pass != Pubkey::default(),
            ErrorCode::MustBeVerifiedHuman
        );
        
        // CRITICAL SECURITY: EXACTLY ONE business per human
        // This prevents:
        // - Storage bloat attacks (10,000 fake businesses)
        // - Chain state explosion
        // - Marketplace spam
        // - Performance degradation
        // Legitimate users can add unlimited divisions OFF-CHAIN
        require!(
            user.business_profile.is_none(),
            ErrorCode::BusinessAlreadyExists
        );
        
        // Validate metadata URI (IPFS CID or Arweave hash)
        require!(
            business_info.metadata_uri.len() <= MAX_METADATA_URI_LENGTH,
            ErrorCode::MetadataUriTooLong
        );
        
        // Create single business profile
        user.business_profile = Some(BusinessProfile {
            name: business_info.name,
            category: business_info.category,
            tax_id: business_info.tax_id,
            metadata_uri: business_info.metadata_uri,  // Points to off-chain data
            created_at: Clock::get()?.unix_timestamp,
            is_verified: false,  // DAO can verify later
            is_active: true,
            volume_processed: 0,
            employee_count: 0,
            last_metadata_update: Clock::get()?.unix_timestamp,
        });
        
        // Update protocol
        let protocol_mut = &mut ctx.accounts.protocol_state;
        protocol_mut.total_businesses += 1;
        
        emit!(BusinessRegistered {
            human_owner: user.owner,
            business_name: business_info.name.clone(),
            category: business_info.category,
            metadata_uri: business_info.metadata_uri,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        msg!("Business registered. Add divisions/branches via metadata URI update.");
        
        Ok(())
    }

    // ========================================================================
    // BUSINESS METADATA UPDATE - For Off-chain Registry
    // ========================================================================

    /// Updates the off-chain metadata URI for a business.
    /// 
    /// This allows businesses to update their divisions, employees, and locations
    /// without on-chain storage costs. The actual data lives on IPFS or Arweave.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user state
    /// * `new_metadata_uri` - New IPFS CID or Arweave hash (max 128 chars)
    /// 
    /// # Rate Limiting
    /// 
    /// Metadata can only be updated once per day to prevent spam.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Metadata URI updated successfully
    /// * `Err(NoBusinessRegistered)` - User has no business profile
    /// * `Err(MetadataUriTooLong)` - URI exceeds 128 characters
    /// * `Err(MetadataUpdateTooSoon)` - Must wait 24 hours between updates
    /// 
    /// # Events
    /// 
    /// Emits `BusinessMetadataUpdated` with old and new URIs.
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// // Add a new division to business
    /// // 1. Upload new JSON to IPFS with additional division
    /// // 2. Update on-chain pointer
    /// update_business_metadata(ctx, "ipfs://QmNewHash...")?;
    /// ```
    pub fn update_business_metadata(
        ctx: Context<UpdateBusiness>,
        new_metadata_uri: String,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Must have a business
        require!(
            user.business_profile.is_some(),
            ErrorCode::NoBusinessRegistered
        );
        
        // Validate metadata URI
        require!(
            new_metadata_uri.len() <= MAX_METADATA_URI_LENGTH,
            ErrorCode::MetadataUriTooLong
        );
        
        // Rate limit metadata updates (once per day)
        let business = user.business_profile.as_mut().unwrap();
        require!(
            current_time - business.last_metadata_update >= SECONDS_PER_DAY,
            ErrorCode::MetadataUpdateTooSoon
        );
        
        // Update metadata pointer
        let old_uri = business.metadata_uri.clone();
        business.metadata_uri = new_metadata_uri.clone();
        business.last_metadata_update = current_time;
        
        emit!(BusinessMetadataUpdated {
            business_owner: user.owner,
            business_name: business.name.clone(),
            old_metadata_uri: old_uri,
            new_metadata_uri,
            timestamp: current_time,
        });
        
        msg!("Metadata updated. Off-chain divisions/branches can be modified freely.");
        
        Ok(())
    }

    // ========================================================================
    // BUSINESS REMOVAL - Clean Deregistration
    // ========================================================================

    /// Removes a business profile from a user account.
    /// 
    /// This cleanly deregisters a business, freeing the slot for a new business
    /// if desired. All off-chain data should be handled separately.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user state and protocol state
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Business removed successfully
    /// * `Err(NoBusinessRegistered)` - User has no business to remove
    /// 
    /// # Events
    /// 
    /// Emits `BusinessRemoved` with owner and business name.
    /// 
    /// # Note
    /// 
    /// After removal, user can register a new business. Off-chain metadata
    /// (IPFS/Arweave) should be updated separately if needed.
    pub fn remove_business(ctx: Context<RemoveBusiness>) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        
        // Must have a business
        require!(
            user.business_profile.is_some(),
            ErrorCode::NoBusinessRegistered
        );
        
        let business = user.business_profile.take().unwrap();
        
        // Update protocol
        let protocol = &mut ctx.accounts.protocol_state;
        if protocol.total_businesses > 0 {
            protocol.total_businesses -= 1;
        }
        
        emit!(BusinessRemoved {
            human_owner: user.owner,
            business_name: business.name,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    // ========================================================================
    // MERCHANT STATUS UPDATE - Simple OR Logic
    // ========================================================================

    /// Updates merchant status based on 30-day rolling metrics.
    /// 
    /// Merchant status is automatically determined by transaction patterns.
    /// Users qualify via EITHER unique customers OR monthly volume (OR logic),
    /// ensuring both retail and service merchants can qualify.
    /// 
    /// # Qualification Thresholds (30-day period)
    /// 
    /// | Tier | Unique Customers | OR | Monthly Volume |
    /// |------|------------------|-----|----------------|
    /// | Emerging | â‰¥10 | OR | â‰¥1,000 AGORA |
    /// | Small | â‰¥25 | OR | â‰¥5,000 AGORA |
    /// | Medium | â‰¥100 | OR | â‰¥25,000 AGORA |
    /// | Large | â‰¥500 | OR | â‰¥100,000 AGORA |
    /// | Enterprise | â‰¥2,000 | OR | â‰¥500,000 AGORA |
    /// 
    /// # Fee Discounts
    /// 
    /// | Tier | Discount |
    /// |------|----------|
    /// | Emerging | 25% off â†’ 0.0375% |
    /// | Small | 50% off â†’ 0.025% |
    /// | Medium | 75% off â†’ 0.0125% |
    /// | Large/Enterprise | FREE â†’ 0% |
    /// 
    /// # Decay Mechanism
    /// 
    /// 90 days without merchant transactions causes tier to drop by one level.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user state and protocol state
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Status updated (or decayed) successfully
    /// 
    /// # Events
    /// 
    /// Emits `MerchantStatusUpgraded` on tier increase, `MerchantStatusDecayed` on decay.
    /// 
    /// # Note
    /// 
    /// All thresholds are DAO-adjustable to adapt to real economy pricing.
    pub fn update_merchant_status(ctx: Context<UpdateMerchantStatus>) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Reset monthly period if 30 days passed
        maybe_reset_monthly_period(user, current_time);
        
        // Check for decay first (90 days inactivity)
        let days_inactive = (current_time - user.last_merchant_tx) / SECONDS_PER_DAY;
        if days_inactive > MERCHANT_DECAY_DAYS {
            decay_merchant_status(user)?;
            return Ok(());
        }
        
        // Use 30-day rolling metrics for tier calculation
        let unique_customers = user.monthly_unique_count;
        let monthly_volume = user.monthly_volume_received;
        
        // Determine tier using OR logic
        let new_tier = determine_merchant_tier(unique_customers, monthly_volume);
        
        // Upgrade if improved (never auto-downgrade, only decay)
        if new_tier > user.merchant_status {
            let previous_tier = user.merchant_status.clone();
            user.merchant_status = new_tier.clone();
            user.last_merchant_tx = current_time;
            
            // Only increment total_merchants on first merchant status
            if previous_tier == MerchantTier::None {
                let protocol = &mut ctx.accounts.protocol_state;
                protocol.total_merchants += 1;
            }
            
            emit!(MerchantStatusUpgraded {
                merchant: user.owner,
                new_tier,
                unique_customers,
                monthly_volume,
                timestamp: current_time,
            });
        }
        
        Ok(())
    }

    // ========================================================================
    // BUSINESS TRANSACTIONS - Through Human Wallet
    // ========================================================================

    /// Executes a business transaction with merchant-tier fee discounts.
    /// 
    /// Business transactions flow through the human owner's wallet but
    /// are tracked separately for merchant analytics. Uses sender's merchant
    /// tier to determine fee discount.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing sender, recipient, and token accounts
    /// * `amount` - Amount of AGORA to transfer (in base units, 9 decimals)
    /// * `transaction_metadata` - Optional invoice number or reference string
    /// 
    /// # Fee Calculation
    /// 
    /// Fee is based on sender's merchant tier:
    /// - Enterprise/Large: FREE (0%)
    /// - Medium: 75% off (0.0125%)
    /// - Small: 50% off (0.025%)
    /// - Emerging: 25% off (0.0375%)
    /// - None: Standard fee (0.05%)
    /// 
    /// Fee distribution: 50% burned, 50% to treasury
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Transaction completed successfully
    /// * `Err(NoBusinessRegistered)` - Sender has no business profile
    /// * `Err(BusinessNotActive)` - Sender's business is deactivated
    /// 
    /// # Events
    /// 
    /// Emits `BusinessTransactionCompleted` with transaction details.
    /// 
    /// # Side Effects
    /// 
    /// - Updates sender's volume_sent and business volume_processed
    /// - Updates recipient's volume_received and unique_payers
    /// - Updates recipient's 30-day rolling metrics for merchant detection
    pub fn business_transaction(
        ctx: Context<BusinessTransaction>,
        amount: u64,
        transaction_metadata: Option<String>,  // Optional invoice/reference
    ) -> Result<()> {
        let sender = &ctx.accounts.sender_state;
        let recipient = &mut ctx.accounts.recipient_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Verify sender has active business
        require!(
            sender.business_profile.is_some(),
            ErrorCode::NoBusinessRegistered
        );
        
        let business = sender.business_profile.as_ref().unwrap();
        require!(business.is_active, ErrorCode::BusinessNotActive);
        
        // Calculate fee based on human's merchant status
        let fee_rate = calculate_merchant_fee(&sender.merchant_status);
        let fee_amount = (amount * fee_rate) / FEE_DIVISOR;
        let transfer_amount = amount - fee_amount;
        
        // Store sender owner before mutable borrow
        let sender_owner = sender.owner;
        
        // Update sender lifetime metrics
        let sender_mut = &mut ctx.accounts.sender_state;
        sender_mut.volume_sent += amount as u128;
        sender_mut.tx_count_sent += 1;
        
        if let Some(business) = &mut sender_mut.business_profile {
            business.volume_processed += amount as u128;
        }
        
        // Update recipient lifetime stats
        recipient.volume_received += amount as u128;
        recipient.tx_count_received += 1;
        
        // Track lifetime unique payers
        if !recipient.unique_payers.contains(&sender_owner) {
            recipient.unique_payers.push(sender_owner);
            recipient.unique_payers_count += 1;
        }
        
        // Update 30-day rolling volume for merchant detection
        update_monthly_volume(recipient, &sender_owner, amount, current_time);
        
        // Execute transfer
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            transfer_amount,
        )?;
        
        // Handle fees
        if fee_amount > 0 {
            handle_fees(&ctx, fee_amount)?;
        }
        
        emit!(BusinessTransactionCompleted {
            human_owner: sender_owner,
            business_name: business.name.clone(),
            recipient: recipient.owner,
            amount: transfer_amount,
            fee: fee_amount,
            metadata: transaction_metadata,
            timestamp: current_time,
        });
        
        Ok(())
    }

    // ========================================================================
    // REGULAR TRANSFERS - For All Users
    // ========================================================================

    /// Transfers AGORA tokens with activity-based fees.
    /// 
    /// This is the standard transfer function for all users. Fees are calculated
    /// based on the sender's transaction activity - active users pay less,
    /// dormant users pay more. This encourages using AGORA as currency.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing sender, recipient, token accounts, and protocol state
    /// * `amount` - Amount of AGORA to transfer (in base units, 9 decimals)
    /// 
    /// # Fee Calculation
    /// 
    /// For regular users (non-merchants):
    /// 
    /// | Activity | Condition | Fee |
    /// |----------|-----------|-----|
    /// | Active | TX within 7 days | 0.04% (0.8Ã—) |
    /// | Normal | TX within 30 days | 0.05% (1.0Ã—) |
    /// | Inactive | TX within 90 days | 0.075% (1.5Ã—) |
    /// | Dormant | No TX for 90+ days | 0.10% (2.0Ã—) |
    /// 
    /// For merchants: Uses tier-based fee (see `update_merchant_status`)
    /// 
    /// Fee distribution: 50% burned (deflationary), 50% to treasury
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Transfer completed successfully
    /// * `Err(UserBlacklisted)` - Sender is permanently blacklisted
    /// * `Err(UserSuspended)` - Sender is temporarily suspended
    /// * `Err(TransactionTooSoon)` - Must wait 60 seconds between transactions
    /// 
    /// # Events
    /// 
    /// Emits `TransferCompleted` with amounts, fee, and activity status.
    /// 
    /// # Side Effects
    /// 
    /// - Updates sender's last_tx_timestamp and volume_sent
    /// - Updates recipient's volume_received and unique_payers
    /// - Updates recipient's 30-day rolling metrics
    /// - Burns 50% of fee, sends 50% to treasury
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// // Transfer 100 AGORA
    /// transfer_with_fee(ctx, 100_000_000_000)?;
    /// // If sender is active (TX within 7 days): 0.04% fee = 0.04 AGORA
    /// // If sender is dormant (90+ days): 0.10% fee = 0.10 AGORA
    /// ```
    pub fn transfer_with_fee(
        ctx: Context<TransferWithFee>,
        amount: u64,
    ) -> Result<()> {
        let sender = &mut ctx.accounts.sender_state;
        let recipient = &mut ctx.accounts.recipient_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Security checks
        require!(!sender.permanently_blacklisted, ErrorCode::UserBlacklisted);
        require!(current_time >= sender.suspended_until, ErrorCode::UserSuspended);
        
        // =====================================================================
        // CHILD PROTECTION: Can only spend accessible_balance
        // =====================================================================
        if sender.is_child {
            // Check if child turned 18
            let current_age_days = ((current_time - sender.birth_timestamp) / SECONDS_PER_DAY) as u64;
            if current_age_days >= CHILD_ADULT_AGE_DAYS {
                // Unlock everything
                let unlocked = sender.locked_balance;  // Save before zeroing
                sender.accessible_balance += sender.locked_balance;
                sender.locked_balance = 0;
                sender.is_child = false;
                
                emit!(ChildTurned18 {
                    user: sender.owner,
                    unlocked_amount: unlocked,
                    timestamp: current_time,
                });
            } else {
                // Still a child - can only spend accessible balance
                require!(
                    amount <= sender.accessible_balance,
                    ErrorCode::TransferExceedsAccessibleBalance
                );
            }
        }
        
        // Rate limiting
        require!(
            current_time - sender.last_tx_timestamp >= MIN_TIME_BETWEEN_TX,
            ErrorCode::TransactionTooSoon
        );
        
        // Calculate activity-based fee
        let fee_amount = calculate_transfer_fee(sender, amount, current_time);
        let transfer_amount = amount - fee_amount;
        
        // Update sender balances (deduct from accessible first)
        if sender.is_child {
            sender.accessible_balance -= amount;
        }
        
        // Update sender lifetime stats
        sender.volume_sent += amount as u128;
        sender.tx_count_sent += 1;
        sender.last_tx_timestamp = current_time;
        
        // Update recipient lifetime stats
        recipient.volume_received += amount as u128;
        recipient.tx_count_received += 1;
        
        // Track lifetime unique payers
        if !recipient.unique_payers.contains(&sender.owner) {
            recipient.unique_payers.push(sender.owner);
            recipient.unique_payers_count += 1;
        }
        
        // Update 30-day rolling volume for merchant detection
        update_monthly_volume(recipient, &sender.owner, amount, current_time);
        
        // Execute transfer
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            transfer_amount,
        )?;
        
        // Handle fees (50% burn, 50% treasury)
        if fee_amount > 0 {
            let burn_amount = fee_amount / 2;
            let treasury_amount = fee_amount - burn_amount;
            
            // Burn 50%
            token::burn(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Burn {
                        mint: ctx.accounts.mint.to_account_info(),
                        from: ctx.accounts.from.to_account_info(),
                        authority: ctx.accounts.signer.to_account_info(),
                    },
                ),
                burn_amount,
            )?;
            
            // Treasury 50%
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.from.to_account_info(),
                        to: ctx.accounts.treasury_account.to_account_info(),
                        authority: ctx.accounts.signer.to_account_info(),
                    },
                ),
                treasury_amount,
            )?;
            
            // Update protocol stats
            let protocol = &mut ctx.accounts.protocol_state;
            protocol.total_burned += burn_amount as u128;
        }
        
        emit!(TransferCompleted {
            sender: sender.owner,
            recipient: recipient.owner,
            amount: transfer_amount,
            fee: fee_amount,
            activity_status: get_activity_status(sender.last_tx_timestamp, current_time),
            timestamp: current_time,
        });
        
        Ok(())
    }

    // ========================================================================
    // EMPLOYEE MANAGEMENT - Simplified
    // ========================================================================

    /// Updates the employee count for a business.
    /// 
    /// This stores only the count on-chain for analytics. Full employee details
    /// (wallets, names, roles) should be managed in off-chain metadata.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user state
    /// * `employee_count` - Number of employees in the business
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Employee count updated
    /// * `Err(NoBusinessRegistered)` - User has no business profile
    /// 
    /// # Events
    /// 
    /// Emits `EmployeeCountUpdated` with business name and new count.
    /// 
    /// # Note
    /// 
    /// For detailed employee management (payment splitting, roles, permissions),
    /// use the off-chain metadata system (IPFS/Arweave).
    pub fn set_employee_count(
        ctx: Context<UpdateBusiness>,
        employee_count: u32,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        
        require!(
            user.business_profile.is_some(),
            ErrorCode::NoBusinessRegistered
        );
        
        // Update on-chain employee count (actual list is off-chain)
        let business = user.business_profile.as_mut().unwrap();
        business.employee_count = employee_count;
        
        emit!(EmployeeCountUpdated {
            business_owner: user.owner,
            business_name: business.name.clone(),
            employee_count,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        msg!("Employee count updated. Manage employee details in off-chain metadata.");
        
        Ok(())
    }

    // ========================================================================
    // DAILY CLAIMS - HUMANS ONLY
    // ========================================================================

    /// Claims accumulated AGORA tokens for a verified human.
    /// 
    /// This is the core UBI distribution function. Every verified human accumulates
    /// 100 AGORA per day, up to 30 days maximum. Only humans can claim - businesses
    /// cannot claim UBI directly.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user state, mint, and token accounts
    /// 
    /// # Accumulation Rules
    /// 
    /// - **Daily Rate**: 100 AGORA per day
    /// - **Max Accumulation**: 30 days (3,000 AGORA)
    /// - **Claim Frequency**: Sponsors can claim daily; non-contributors monthly
    /// 
    /// # Liveness Requirements
    /// 
    /// Annual liveness verification is required to continue claiming:
    /// 
    /// | Status | Effect |
    /// |--------|--------|
    /// | Active | Normal claiming |
    /// | Warning | Claim works, warning emitted (60 days before expiry) |
    /// | Expired | Claim works, urgent warning (30 day grace period) |
    /// | Suspended | Cannot claim until liveness verified |
    /// | Locked | Temporarily locked from too many failed attempts |
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Tokens minted to user's account
    /// * `Err(OnlyHumansCanClaim)` - Caller is not a verified human
    /// * `Err(UserBlacklisted)` - User is permanently blacklisted
    /// * `Err(UserSuspended)` - User is temporarily suspended
    /// * `Err(LivenessVerificationRequired)` - Must complete annual liveness check
    /// * `Err(NothingToClaim)` - No tokens accumulated
    /// * `Err(ClaimTooSoon)` - Must wait for claim interval
    /// 
    /// # Events
    /// 
    /// Emits `DailyClaimed` with amount and timestamp.
    /// May also emit `LivenessWarning` or `LivenessExpiredWarning`.
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// // User hasn't claimed for 10 days
    /// claim_daily(ctx)?;
    /// // User receives 1,000 AGORA (10 days Ã— 100)
    /// 
    /// // User hasn't claimed for 60 days
    /// claim_daily(ctx)?;
    /// // User receives 3,000 AGORA (capped at 30 days)
    /// ```
    pub fn claim_daily(ctx: Context<ClaimDaily>) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;

        // CRITICAL: Only humans can claim UBI
        require!(
            user.is_human,
            ErrorCode::OnlyHumansCanClaim
        );

        // Businesses CANNOT claim - prevents Sybil attacks
        // Each business needs a human owner who claims personally

        // Security checks - INDIVIDUAL penalties only
        require!(
            !user.permanently_blacklisted,
            ErrorCode::UserBlacklisted
        );
        require!(
            current_time >= user.suspended_until,
            ErrorCode::UserSuspended
        );

        // =====================================================================
        // ANNUAL LIVENESS CHECK - Must prove you're alive to claim
        // =====================================================================
        let liveness_status = get_liveness_status(&user.liveness, current_time);

        match liveness_status {
            LivenessStatus::Active => {
                // All good, can claim
            },
            LivenessStatus::Warning => {
                emit!(LivenessWarning {
                    user: ctx.accounts.user.key(),
                    days_until_expiry: (user.liveness.next_verification_due - current_time) / SECONDS_PER_DAY,
                    timestamp: current_time,
                });
            },
            LivenessStatus::Expired => {
                let grace_remaining = (user.liveness.next_verification_due + LIVENESS_GRACE_PERIOD - current_time) / SECONDS_PER_DAY;
                emit!(LivenessExpiredWarning {
                    user: ctx.accounts.user.key(),
                    grace_days_remaining: grace_remaining,
                    timestamp: current_time,
                });
            },
            LivenessStatus::Suspended => {
                return Err(ErrorCode::LivenessVerificationRequired.into());
            },
            LivenessStatus::Locked => {
                return Err(ErrorCode::LivenessVerificationLocked.into());
            },
        }

        // Calculate claimable amount - NO country multiplier, full UBI for everyone
        let claimable = calculate_claimable_amount(user, current_time)?;
        require!(claimable > 0, ErrorCode::NothingToClaim);

        // =====================================================================
        // COUNTRY SANCTION CHECK
        // =====================================================================
        // If user's country is under DAO sanction, reduce their UBI.
        // Citizens still receive reduced amount - they are victims too.
        // Sanction account is optional - if not provided, no sanction applies.
        let final_claimable = if let Some(sanction_account) = &ctx.accounts.country_sanction {
            let sanction = sanction_account;
            
            // Check if sanction applies to this user's country
            if sanction.is_active 
                && sanction.country_code == user.citizenship 
                && current_time < sanction.end_timestamp 
            {
                // Apply sanction rate (e.g., 1000 = 10% of normal)
                let sanctioned_amount = (claimable * sanction.sanction_rate) / SANCTION_RATE_DIVISOR;
                
                emit!(SanctionedClaimReduced {
                    user: ctx.accounts.user.key(),
                    country_code: user.citizenship,
                    original_amount: claimable,
                    reduced_amount: sanctioned_amount,
                    sanction_rate: sanction.sanction_rate,
                    timestamp: current_time,
                });
                
                sanctioned_amount
            } else {
                claimable
            }
        } else {
            claimable
        };

        // Check claim frequency
        let min_interval = determine_claim_interval(user);
        require!(
            current_time - user.last_claim_timestamp >= min_interval,
            ErrorCode::ClaimTooSoon
        );
        
        // =====================================================================
        // CHILD PROTECTION: Split claim into accessible and locked pools
        // =====================================================================
        // First, check if child has turned 18 since registration
        let current_age_days = ((current_time - user.birth_timestamp) / SECONDS_PER_DAY) as u64;
        
        // If child just turned 18, unlock everything and mark as adult
        if user.is_child && current_age_days >= CHILD_ADULT_AGE_DAYS {
            let unlocked = user.locked_balance;  // Save before zeroing
            user.accessible_balance += user.locked_balance;
            user.locked_balance = 0;
            user.is_child = false;
            
            emit!(ChildTurned18 {
                user: ctx.accounts.user.key(),
                unlocked_amount: unlocked,
                timestamp: current_time,
            });
        }
        
        // Calculate accessible and locked portions
        let (accessible_portion, locked_portion) = if user.is_child {
            let rate = if current_age_days >= CHILD_MATURITY_BOOST_AGE_DAYS {
                CHILD_RATE_16_TO_18  // φ/(π×e) ≈ 18.92%
            } else {
                CHILD_RATE_UNDER_16  // 1/(π×e) ≈ 11.69%
            };
            let accessible = (final_claimable * rate) / CHILD_RATE_DIVISOR;
            let locked = final_claimable - accessible;
            (accessible, locked)
        } else {
            (final_claimable, 0)  // Adults get 100% accessible
        };
        
        // Mint tokens
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.protocol_state.to_account_info(),
                },
                &[&[b"protocol", &[ctx.accounts.protocol_state.bump]]],
            ),
            final_claimable,
        )?;
        
        // Update state
        user.last_claim_timestamp = current_time;
        user.total_claimed += final_claimable as u128;
        user.accessible_balance += accessible_portion;
        user.locked_balance += locked_portion;

        let protocol = &mut ctx.accounts.protocol_state;
        protocol.total_minted += final_claimable as u128;

        emit!(DailyClaimed {
            user: ctx.accounts.user.key(),
            amount: final_claimable,
            accessible_portion,
            locked_portion,
            is_child: user.is_child,
            timestamp: current_time,
        });

        Ok(())
    }

    // ========================================================================
    // COUNTRY SANCTIONS - DAO Governance
    // ========================================================================

    /// Initializes the sanction registry. Must be called once before any sanctions.
    pub fn initialize_sanction_registry(ctx: Context<InitializeSanctionRegistry>) -> Result<()> {
        let registry = &mut ctx.accounts.sanction_registry;
        registry.total_active_sanctions = 0;
        registry.total_historical_sanctions = 0;
        registry.last_updated = Clock::get()?.unix_timestamp;
        registry.bump = ctx.bumps.sanction_registry;
        
        Ok(())
    }

    /// Imposes a sanction on a country for committing atrocities.
    /// 
    /// This function can only be called by the DAO after a successful vote.
    /// Sanctions reduce UBI for citizens of the offending country.
    /// 
    /// # Philosophy
    /// 
    /// When governments commit genocide, war crimes, or mass civilian harm,
    /// there must be consequences. Traditional sanctions hurt citizens while
    /// leaders stay comfortable. AGORA sanctions are different:
    /// 
    /// - Citizens still receive reduced UBI (never zero) - they are victims too
    /// - Sanctions are temporary - pressure for change, not permanent punishment  
    /// - Democratic decision via DAO - not unilateral
    /// - Fully transparent on-chain
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing sanction account and DAO authority
    /// * `country_code` - ISO 3166-1 alpha-3 code (e.g., "USA", "RUS", "ISR")
    /// * `sanction_rate` - Percentage of normal UBI to pay (1000 = 10%)
    /// * `duration_days` - How long the sanction lasts
    /// * `reason` - Brief description of the atrocity
    /// * `evidence_hash` - IPFS hash of evidence documentation
    /// * `proposal_id` - DAO proposal ID that approved this sanction
    /// * `votes_for` - Number of votes in favor
    /// * `votes_against` - Number of votes against
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Sanction imposed successfully
    /// * `Err(Unauthorized)` - Caller is not DAO authority
    /// * `Err(SanctionRateTooLow)` - Rate below minimum (5%)
    /// * `Err(SanctionDurationTooLong)` - Duration exceeds 1 year
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// // DAO votes to sanction country for genocide
    /// impose_country_sanction(
    ///     ctx,
    ///     *b"XXX",           // Country code
    ///     1000,              // 10% of normal UBI
    ///     120,               // 4 months
    ///     "Genocide against civilian population".to_string(),
    ///     evidence_ipfs_hash,
    ///     42,                // Proposal ID
    ///     850000,            // Votes for
    ///     150000,            // Votes against
    /// )?;
    /// ```
    pub fn impose_country_sanction(
        ctx: Context<ImposeCountrySanction>,
        country_code: [u8; 3],
        sanction_rate: u64,
        duration_days: u64,
        reason: String,
        evidence_hash: [u8; 32],
        proposal_id: u64,
        votes_for: u64,
        votes_against: u64,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Only DAO authority can impose sanctions
        require!(
            ctx.accounts.authority.key() == ctx.accounts.protocol_state.authority,
            ErrorCode::Unauthorized
        );
        
        // Validate sanction parameters
        require!(
            sanction_rate >= MIN_SANCTION_RATE,
            ErrorCode::SanctionRateTooLow
        );
        require!(
            reason.len() <= 256,
            ErrorCode::SanctionReasonTooLong
        );
        
        let duration_seconds = (duration_days as i64) * SECONDS_PER_DAY;
        require!(
            duration_seconds <= MAX_SANCTION_DURATION,
            ErrorCode::SanctionDurationTooLong
        );
        
        // Create sanction record
        let sanction = &mut ctx.accounts.country_sanction;
        sanction.country_code = country_code;
        sanction.is_active = true;
        sanction.sanction_rate = sanction_rate;
        sanction.reason = reason.clone();
        sanction.evidence_hash = evidence_hash;
        sanction.start_timestamp = current_time;
        sanction.end_timestamp = current_time + duration_seconds;
        sanction.dao_proposal_id = proposal_id;
        sanction.imposed_by = ctx.accounts.authority.key();
        sanction.vote_count_for = votes_for;
        sanction.vote_count_against = votes_against;
        sanction.bump = ctx.bumps.country_sanction;
        
        // Update registry
        let registry = &mut ctx.accounts.sanction_registry;
        registry.total_active_sanctions += 1;
        registry.total_historical_sanctions += 1;
        registry.last_updated = current_time;
        
        emit!(CountrySanctionImposed {
            country_code,
            sanction_rate,
            duration_days,
            reason,
            evidence_hash,
            proposal_id,
            votes_for,
            votes_against,
            start_timestamp: current_time,
            end_timestamp: sanction.end_timestamp,
        });
        
        Ok(())
    }

    /// Lifts a country sanction early (before expiration).
    /// 
    /// Can be called by DAO if the country has changed behavior or
    /// if the sanction was imposed in error.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing sanction account and DAO authority
    /// * `country_code` - ISO 3166-1 alpha-3 code
    /// * `lift_reason` - Why the sanction is being lifted early
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Sanction lifted successfully
    /// * `Err(Unauthorized)` - Caller is not DAO authority
    /// * `Err(SanctionNotActive)` - No active sanction for this country
    pub fn lift_country_sanction(
        ctx: Context<LiftCountrySanction>,
        country_code: [u8; 3],
        lift_reason: String,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Only DAO authority can lift sanctions
        require!(
            ctx.accounts.authority.key() == ctx.accounts.protocol_state.authority,
            ErrorCode::Unauthorized
        );
        
        let sanction = &mut ctx.accounts.country_sanction;
        require!(sanction.is_active, ErrorCode::SanctionNotActive);
        
        // Deactivate sanction
        sanction.is_active = false;
        
        // Update registry
        let registry = &mut ctx.accounts.sanction_registry;
        if registry.total_active_sanctions > 0 {
            registry.total_active_sanctions -= 1;
        }
        registry.last_updated = current_time;
        
        emit!(CountrySanctionLifted {
            country_code,
            lift_reason,
            original_end_timestamp: sanction.end_timestamp,
            actual_end_timestamp: current_time,
        });
        
        Ok(())
    }

    // ========================================================================
    // EMERGENCY CONTROLS
    // ========================================================================

    /// Emergency function to freeze or unfreeze business registration.
    /// 
    /// This allows the protocol authority (later DAO) to halt new business
    /// registrations if a spam attack or vulnerability is detected.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing protocol state and authority
    /// * `freeze` - `true` to freeze, `false` to unfreeze
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Freeze state updated
    /// * `Err(Unauthorized)` - Caller is not the protocol authority
    /// 
    /// # Events
    /// 
    /// Emits `BusinessRegistrationFrozen` with new state and authority.
    /// 
    /// # Security
    /// 
    /// Only the protocol authority can call this. After launch, authority
    /// should be transferred to DAO multisig.
    pub fn freeze_business_registration(
        ctx: Context<EmergencyControl>,
        freeze: bool,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.protocol_state.authority,
            ErrorCode::Unauthorized
        );

        let protocol = &mut ctx.accounts.protocol_state;
        protocol.business_registration_frozen = freeze;

        emit!(BusinessRegistrationFrozen {
            frozen: freeze,
            authority: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    // ========================================================================
    // DAO PROPOSAL SYSTEM
    // ========================================================================
    // Governance through proposals with bond mechanism.
    // Anyone can propose, but spam costs money.
    // Bond returned if proposal reaches 50% of quorum.

    /// Create a new DAO proposal.
    /// 
    /// # Bond Mechanism
    /// 
    /// Proposer must transfer AGORA tokens as bond:
    /// - Standard: 100 AGORA
    /// - Treasury: 500 AGORA  
    /// - Constitutional: 1000 AGORA
    /// - Sanction: 2000 AGORA
    /// 
    /// Bond is returned if proposal reaches >= 50% of quorum.
    /// Bond is forfeited if proposal fails to generate interest.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context with proposal account, proposer, registry
    /// * `proposal_type` - Type of proposal (determines parameters)
    /// * `title` - Short title (max 64 bytes)
    /// * `description_hash` - IPFS hash of full proposal text
    /// * `treasury_amount` - For treasury proposals: amount to spend
    /// * `treasury_recipient` - For treasury proposals: recipient
    /// * `sanction_country` - For sanction proposals: ISO country code
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal_type: ProposalType,
        title: [u8; 64],
        description_hash: [u8; 32],
        treasury_amount: u64,
        treasury_recipient: Pubkey,
        sanction_country: [u8; 3],
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let proposer_state = &mut ctx.accounts.proposer_state;
        
        // Check if proposer is banned from creating proposals
        require!(
            proposer_state.proposal_reputation > REP_THRESHOLD_BAN,
            ErrorCode::ProposerBanned
        );
        
        // Determine base bond amount and voting parameters based on type
        let (base_bond, quorum, approval_threshold, voting_period) = match proposal_type {
            ProposalType::Standard => (
                PROPOSAL_BOND_STANDARD,
                QUORUM_STANDARD,
                APPROVAL_STANDARD,
                VOTING_PERIOD_STANDARD,
            ),
            ProposalType::Treasury => (
                PROPOSAL_BOND_TREASURY,
                QUORUM_TREASURY,
                APPROVAL_TREASURY,
                VOTING_PERIOD_TREASURY,
            ),
            ProposalType::Constitutional => (
                PROPOSAL_BOND_CONSTITUTIONAL,
                QUORUM_CONSTITUTIONAL,
                APPROVAL_CONSTITUTIONAL,
                VOTING_PERIOD_CONSTITUTIONAL,
            ),
            ProposalType::Sanction => (
                PROPOSAL_BOND_SANCTION,
                QUORUM_SANCTION,
                APPROVAL_SANCTION,
                VOTING_PERIOD_SANCTION,
            ),
        };

        // Apply bond multiplier based on reputation: 1 + (abs(reputation) / 2)
        let bond_multiplier: u64 = if proposer_state.proposal_reputation >= 0 {
            1
        } else {
            1 + (proposer_state.proposal_reputation.abs() / 2) as u64
        };
        
        let bond_amount = base_bond * bond_multiplier;

        // Transfer bond from proposer to proposal escrow
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.proposer_token_account.to_account_info(),
                to: ctx.accounts.bond_escrow.to_account_info(),
                authority: ctx.accounts.proposer.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, bond_amount)?;

        // Update proposer stats
        proposer_state.proposals_created += 1;

        // Get and increment proposal ID
        let registry = &mut ctx.accounts.proposal_registry;
        let proposal_id = registry.next_proposal_id;
        registry.next_proposal_id += 1;
        registry.total_proposals += 1;
        registry.active_proposals += 1;

        // Initialize proposal
        let proposal = &mut ctx.accounts.proposal;
        proposal.id = proposal_id;
        proposal.proposer = ctx.accounts.proposer.key();
        proposal.proposal_type = proposal_type;
        proposal.status = ProposalStatus::Active;
        proposal.title = title;
        proposal.description_hash = description_hash;
        proposal.bond_amount = bond_amount;
        proposal.bond_resolved = false;
        proposal.votes_yes = 0;
        proposal.votes_no = 0;
        proposal.votes_abstain = 0;
        proposal.total_voters = 0;
        proposal.quorum_required = quorum;
        proposal.approval_threshold = approval_threshold;
        proposal.created_at = current_time;
        proposal.voting_ends_at = current_time + voting_period;
        proposal.executed_at = 0;
        proposal.treasury_amount = treasury_amount;
        proposal.treasury_recipient = treasury_recipient;
        proposal.sanction_country = sanction_country;
        proposal.bump = ctx.bumps.proposal;

        emit!(ProposalCreated {
            proposal_id,
            proposer: ctx.accounts.proposer.key(),
            proposal_type,
            bond_amount,
            voting_ends_at: current_time + voting_period,
        });

        Ok(())
    }

    /// Vote on an active proposal.
    /// 
    /// Each verified human can vote once per proposal.
    /// Votes: 0 = No, 1 = Yes, 2 = Abstain
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context with proposal, voter's user state, vote record
    /// * `choice` - Vote choice (0=no, 1=yes, 2=abstain)
    pub fn vote_on_proposal(
        ctx: Context<VoteOnProposal>,
        choice: u8,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let proposal = &mut ctx.accounts.proposal;

        // Verify proposal is active
        require!(
            proposal.status == ProposalStatus::Active,
            ErrorCode::ProposalNotActive
        );

        // Verify voting period hasn't ended
        require!(
            current_time < proposal.voting_ends_at,
            ErrorCode::VotingEnded
        );

        // Verify voter is a verified human
        require!(
            ctx.accounts.user_state.is_human,
            ErrorCode::NotVerifiedHuman
        );

        // Record vote
        match choice {
            0 => proposal.votes_no += 1,
            1 => proposal.votes_yes += 1,
            2 => proposal.votes_abstain += 1,
            _ => return Err(ErrorCode::InvalidVoteChoice.into()),
        }
        proposal.total_voters += 1;

        // Create vote record (prevents double voting via PDA)
        let vote_record = &mut ctx.accounts.vote_record;
        vote_record.voter = ctx.accounts.voter.key();
        vote_record.proposal_id = proposal.id;
        vote_record.choice = choice;
        vote_record.voted_at = current_time;
        vote_record.bump = ctx.bumps.vote_record;

        emit!(VoteCast {
            proposal_id: proposal.id,
            voter: ctx.accounts.voter.key(),
            choice,
            total_votes: proposal.total_voters,
        });

        Ok(())
    }

    /// Finalize a proposal after voting period ends.
    /// 
    /// Determines outcome and handles bond:
    /// - If >= 50% of quorum reached: bond returned to proposer
    /// - If < 50% of quorum: bond forfeited to treasury
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context with proposal, proposer token account, treasury
    pub fn finalize_proposal(ctx: Context<FinalizeProposal>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let proposal = &mut ctx.accounts.proposal;
        let proposer_state = &mut ctx.accounts.proposer_state;

        // Verify proposal is active
        require!(
            proposal.status == ProposalStatus::Active,
            ErrorCode::ProposalNotActive
        );

        // Verify voting period has ended
        require!(
            current_time >= proposal.voting_ends_at,
            ErrorCode::VotingNotEnded
        );

        let total_votes = proposal.votes_yes + proposal.votes_no;
        let quorum_reached = total_votes >= proposal.quorum_required;
        let approval_reached = if total_votes > 0 {
            (proposal.votes_yes * 10000 / total_votes) >= proposal.approval_threshold
        } else {
            false
        };

        // Calculate quorum percentage for reputation
        let quorum_percentage = if proposal.quorum_required > 0 {
            (total_votes * 10000) / proposal.quorum_required
        } else {
            0
        };

        // Determine outcome and update proposer reputation based on quorum reached
        if quorum_reached && approval_reached {
            // Passed: +2 reputation
            proposal.status = ProposalStatus::Passed;
            proposer_state.proposal_reputation += REP_PROPOSAL_PASSED;
            proposer_state.proposals_passed += 1;
        } else if quorum_reached {
            // Rejected but reached quorum: +1 reputation (legitimate proposal, just unpopular)
            proposal.status = ProposalStatus::Rejected;
            proposer_state.proposal_reputation += REP_PROPOSAL_REJECTED;
            proposer_state.proposals_rejected += 1;
        } else {
            // Expired - reputation penalty based on how much quorum was reached
            proposal.status = ProposalStatus::Expired;
            proposer_state.proposals_expired += 1;
            
            if quorum_percentage >= QUORUM_THRESHOLD_50 {
                // 50%+ of quorum: -1 (close, but not enough interest)
                proposer_state.proposal_reputation += REP_NO_QUORUM_50;
            } else if quorum_percentage >= QUORUM_THRESHOLD_25 {
                // 25-50% of quorum: -2 (low interest)
                proposer_state.proposal_reputation += REP_NO_QUORUM_25;
            } else {
                // <25% of quorum: -3 (obvious spam/irrelevant)
                proposer_state.proposal_reputation += REP_NO_QUORUM_10;
            }
        }

        // Handle bond - only return if quorum reached (50%+)
        let bond_return_threshold = proposal.quorum_required * BOND_REFUND_THRESHOLD / 10000;
        let return_bond = total_votes >= bond_return_threshold;

        let registry = &mut ctx.accounts.proposal_registry;
        registry.active_proposals -= 1;

        if return_bond && !proposal.bond_resolved {
            // Return bond to proposer
            let seeds = &[
                b"bond_escrow",
                &proposal.id.to_le_bytes(),
                &[ctx.bumps.bond_escrow],
            ];
            let signer = &[&seeds[..]];

            let transfer_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.bond_escrow.to_account_info(),
                    to: ctx.accounts.proposer_token_account.to_account_info(),
                    authority: ctx.accounts.bond_escrow.to_account_info(),
                },
                signer,
            );
            token::transfer(transfer_ctx, proposal.bond_amount)?;
            
            registry.total_bonds_returned += proposal.bond_amount;
            proposal.bond_resolved = true;

            emit!(BondReturned {
                proposal_id: proposal.id,
                proposer: proposal.proposer,
                amount: proposal.bond_amount,
            });
        } else if !proposal.bond_resolved {
            // Forfeit bond to treasury
            let seeds = &[
                b"bond_escrow",
                &proposal.id.to_le_bytes(),
                &[ctx.bumps.bond_escrow],
            ];
            let signer = &[&seeds[..]];

            let transfer_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.bond_escrow.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                    authority: ctx.accounts.bond_escrow.to_account_info(),
                },
                signer,
            );
            token::transfer(transfer_ctx, proposal.bond_amount)?;
            
            registry.total_bonds_forfeited += proposal.bond_amount;
            proposal.bond_resolved = true;

            emit!(BondForfeited {
                proposal_id: proposal.id,
                proposer: proposal.proposer,
                amount: proposal.bond_amount,
            });
        }

        emit!(ProposalFinalized {
            proposal_id: proposal.id,
            status: proposal.status,
            votes_yes: proposal.votes_yes,
            votes_no: proposal.votes_no,
            votes_abstain: proposal.votes_abstain,
            bond_returned: return_bond,
        });

        Ok(())
    }

    // ========================================================================
    // GAS POOL MANAGEMENT
    // ========================================================================
    // Manages SOL subsidies for transaction fees.
    // Sponsors deposit SOL â†’ receive tier benefits.
    // Pool subsidizes gas for new users.
    // Anti-abuse system prevents drain attacks.

    /// Initialize the Gas Pool.
    /// 
    /// Called once during protocol setup to create the gas pool PDA.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing gas pool state account
    /// 
    /// # Security
    /// 
    /// Only callable by protocol authority.
    pub fn initialize_gas_pool(ctx: Context<InitializeGasPool>) -> Result<()> {
        let gas_pool = &mut ctx.accounts.gas_pool_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        gas_pool.authority = ctx.accounts.protocol_state.authority;
        gas_pool.total_deposited = 0;
        gas_pool.total_distributed = 0;
        gas_pool.available_balance = 0;
        gas_pool.daily_distributed = 0;
        gas_pool.daily_reset_time = current_time + SECONDS_PER_DAY;
        gas_pool.total_sponsors = 0;
        gas_pool.emergency_paused = false;
        gas_pool.emergency_triggered_at = 0;
        gas_pool.emergency_reason = EmergencyReason::None;
        gas_pool.bump = ctx.bumps.gas_pool_state;
        
        emit!(GasPoolInitialized {
            authority: gas_pool.authority,
            timestamp: current_time,
        });
        
        Ok(())
    }

    /// Sponsor the Gas Pool with SOL.
    /// 
    /// Sponsors deposit SOL and receive tier-based benefits:
    /// - Personal allocation for their own gas fees
    /// - Fee discounts on AGORA transfers
    /// - Priority transaction access
    /// 
    /// # Tier System
    /// 
    /// | Tier | Contribution | Personal % | Fee Discount |
    /// |------|-------------|------------|--------------|
    /// | Bronze | 1 SOL | 20% | 25% |
    /// | Silver | 10 SOL | 15% | 50% |
    /// | Gold | 100 SOL | 10% | 75% |
    /// | Platinum | 1,000 SOL | 5% | 100% |
    /// | Diamond | 10,000 SOL | 3% | 100% |
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing sponsor, gas pool, and user accounts
    /// * `amount` - SOL amount in lamports to deposit
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Sponsorship recorded, tier assigned
    /// * `Err` - Transfer failed or pool paused
    /// 
    /// # Events
    /// 
    /// Emits `SponsorshipReceived` and optionally `SponsorTierUpgraded`.
    pub fn sponsor_gas_pool(
        ctx: Context<SponsorGasPool>,
        amount: u64,
    ) -> Result<()> {
        let gas_pool = &mut ctx.accounts.gas_pool_state;
        let sponsor = &mut ctx.accounts.sponsor_record;
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Check pool not paused
        require!(!gas_pool.emergency_paused, ErrorCode::GasPoolPaused);
        
        // Calculate tier and allocations
        let total_contribution = sponsor.total_contributed + amount;
        let new_tier = calculate_sponsor_tier(total_contribution);
        let personal_pct = get_personal_allocation_pct(&new_tier);
        let personal_amount = (amount * personal_pct) / 100;
        let pool_amount = amount - personal_amount;
        
        // Transfer SOL from sponsor to pool
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.sponsor_wallet.key(),
            &ctx.accounts.gas_pool_vault.key(),
            amount,
        );
        
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.sponsor_wallet.to_account_info(),
                ctx.accounts.gas_pool_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Update sponsor record
        let is_new_sponsor = sponsor.total_contributed == 0;
        let old_tier = sponsor.tier.clone();
        
        sponsor.sponsor = ctx.accounts.sponsor_wallet.key();
        sponsor.tier = new_tier.clone();
        sponsor.total_contributed = total_contribution;
        sponsor.personal_allocation += personal_amount;
        sponsor.pool_contribution += pool_amount;
        
        if is_new_sponsor {
            sponsor.sponsored_at = current_time;
            gas_pool.total_sponsors += 1;
        }
        
        if new_tier != old_tier {
            sponsor.tier_upgraded_at = current_time;
        }
        
        // Update user state with sponsor tier
        user.sponsor_tier = new_tier.clone();
        user.personal_allocation_remaining = sponsor.personal_allocation;
        
        // Update gas pool totals
        gas_pool.total_deposited += amount;
        gas_pool.available_balance += pool_amount;
        
        emit!(SponsorshipReceived {
            sponsor: ctx.accounts.sponsor_wallet.key(),
            amount,
            personal_allocation: personal_amount,
            pool_contribution: pool_amount,
            new_tier: new_tier.clone(),
            timestamp: current_time,
        });
        
        if new_tier != old_tier && !is_new_sponsor {
            emit!(SponsorTierUpgraded {
                sponsor: ctx.accounts.sponsor_wallet.key(),
                from_tier: old_tier,
                to_tier: new_tier,
                total_contributed: total_contribution,
                timestamp: current_time,
            });
        }
        
        Ok(())
    }

    /// Claim a gas subsidy for a transaction.
    /// 
    /// This function is called internally when a user without SOL needs
    /// gas to execute a transaction (claim, transfer, etc.).
    /// 
    /// # Access Tiers
    /// 
    /// | User Type | Daily Limit | Cooldown |
    /// |-----------|-------------|----------|
    /// | Non-contributor | 5 TX | 60 sec |
    /// | Sponsor (personal) | Based on allocation | None |
    /// | Sponsor (depleted) | 5 TX | 60 sec |
    /// 
    /// # Anti-Abuse
    /// 
    /// - Rate limiting per user tier
    /// - Minimum transfer amount for subsidy
    /// - Ping-pong detection
    /// - Emergency brake if pool drains too fast
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user, gas pool, and tracker accounts
    /// * `tx_type` - Type of transaction requesting subsidy
    /// 
    /// # Returns
    /// 
    /// * `Ok(subsidy_amount)` - Lamports transferred to user for gas
    /// * `Err` - Subsidy denied (rate limited, pool paused, etc.)
    pub fn claim_gas_subsidy(
        ctx: Context<ClaimGasSubsidy>,
        tx_type: TransactionType,
    ) -> Result<u64> {
        let gas_pool = &mut ctx.accounts.gas_pool_state;
        let user = &ctx.accounts.user_state;
        let sponsor = &mut ctx.accounts.sponsor_record;
        let tracker = &mut ctx.accounts.tx_tracker;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Check pool not paused
        require!(!gas_pool.emergency_paused, ErrorCode::GasPoolPaused);
        
        // Check pool has funds
        require!(
            gas_pool.available_balance >= MIN_POOL_BALANCE,
            ErrorCode::GasPoolLowBalance
        );
        
        // Reset daily counters if needed
        if current_time >= gas_pool.daily_reset_time {
            gas_pool.daily_distributed = 0;
            gas_pool.daily_reset_time = current_time + SECONDS_PER_DAY;
        }
        
        if current_time >= sponsor.daily_reset_time {
            sponsor.daily_subsidy_count = 0;
            sponsor.daily_reset_time = current_time + SECONDS_PER_DAY;
        }
        
        // Check daily drain limits
        if gas_pool.daily_distributed >= DAILY_DRAIN_EMERGENCY {
            gas_pool.emergency_paused = true;
            gas_pool.emergency_triggered_at = current_time;
            gas_pool.emergency_reason = EmergencyReason::ExcessiveDrain;
            
            emit!(GasPoolEmergency {
                reason: EmergencyReason::ExcessiveDrain,
                daily_distributed: gas_pool.daily_distributed,
                timestamp: current_time,
            });
            
            return Err(ErrorCode::GasPoolEmergencyPause.into());
        }
        
        // Determine subsidy based on user type
        let subsidy_amount: u64;
        let from_personal: bool;
        
        if user.sponsor_tier != SponsorTier::None && sponsor.personal_allocation > 0 {
            // Sponsor with personal allocation remaining
            let gas_cost = estimate_gas_cost(&tx_type);
            
            if sponsor.personal_allocation >= gas_cost {
                sponsor.personal_allocation -= gas_cost;
                subsidy_amount = gas_cost;
                from_personal = true;
            } else {
                // Personal allocation depleted, fall back to pool
                subsidy_amount = apply_free_tier_limits(
                    sponsor.daily_subsidy_count,
                    tracker,
                    current_time,
                )?;
                from_personal = false;
            }
        } else {
            // Non-contributor or depleted sponsor - use free tier
            subsidy_amount = apply_free_tier_limits(
                sponsor.daily_subsidy_count,
                tracker,
                current_time,
            )?;
            from_personal = false;
        }
        
        // Update tracking
        sponsor.daily_subsidy_count += 1;
        
        if !from_personal {
            gas_pool.available_balance -= subsidy_amount;
            gas_pool.total_distributed += subsidy_amount;
            gas_pool.daily_distributed += subsidy_amount;
        }
        
        // Transfer SOL to user (for gas)
        **ctx.accounts.gas_pool_vault.try_borrow_mut_lamports()? -= subsidy_amount;
        **ctx.accounts.user_wallet.try_borrow_mut_lamports()? += subsidy_amount;
        
        emit!(GasSubsidyClaimed {
            user: ctx.accounts.user_wallet.key(),
            amount: subsidy_amount,
            from_personal,
            tx_type,
            timestamp: current_time,
        });
        
        Ok(subsidy_amount)
    }

    /// Record a transaction for anti-abuse tracking.
    /// 
    /// Called after each transaction to update the user's transaction tracker.
    /// Detects suspicious patterns like ping-pong attacks.
    /// 
    /// # Detection Patterns
    /// 
    /// - **Ping-pong**: Aâ†’B followed by Bâ†’A within 5 minutes
    /// - **Rapid fire**: >10 TX in 1 minute
    /// - **Same-recipient abuse**: >5 TX to same address per hour
    /// 
    /// # Penalties
    /// 
    /// - First violation: 30-day suspension
    /// - Second violation: Permanent blacklist
    pub fn record_transaction(
        ctx: Context<RecordTransaction>,
        recipient: Pubkey,
    ) -> Result<()> {
        let tracker = &mut ctx.accounts.tx_tracker;
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Update circular buffer
        tracker.recent_recipients[tracker.buffer_index as usize] = recipient;
        tracker.recent_timestamps[tracker.buffer_index as usize] = current_time;
        tracker.buffer_index = (tracker.buffer_index + 1) % 10;
        
        // Update minute counter
        if current_time - tracker.minute_start > 60 {
            tracker.tx_last_minute = 1;
            tracker.minute_start = current_time;
        } else {
            tracker.tx_last_minute += 1;
        }
        
        // Check for violations
        let mut violation_detected = false;
        let mut violation_type = String::new();
        
        // Check rapid fire
        if tracker.tx_last_minute > RAPID_TX_THRESHOLD {
            violation_detected = true;
            violation_type = "rapid_fire".to_string();
            tracker.suspicion_score = tracker.suspicion_score.saturating_add(30);
        }
        
        // Check ping-pong
        if detect_ping_pong(tracker, &ctx.accounts.user_state.owner, &recipient, current_time) {
            violation_detected = true;
            violation_type = "ping_pong".to_string();
            tracker.suspicion_score = tracker.suspicion_score.saturating_add(50);
        }
        
        // Check same-recipient abuse
        let same_recipient_count = count_same_recipient(tracker, &recipient, current_time, 3600);
        if same_recipient_count > MAX_SAME_RECIPIENT_PER_HOUR {
            tracker.suspicion_score = tracker.suspicion_score.saturating_add(20);
        }
        
        // Apply penalties if violation detected
        if violation_detected {
            user.violations += 1;
            
            if user.violations >= 2 {
                user.permanently_blacklisted = true;
                
                emit!(IndividualBanned {
                    user: user.owner,
                    reason: format!("Second violation: {}", violation_type),
                    timestamp: current_time,
                });
            } else {
                user.suspended_until = current_time + (SUSPENSION_DAYS * SECONDS_PER_DAY);
                
                emit!(IndividualFraudDetected {
                    user: user.owner,
                    fraud_type: violation_type,
                    evidence_hash: [0u8; 32], // Could hash transaction details
                    timestamp: current_time,
                });
            }
        }
        
        Ok(())
    }

    /// Emergency pause the Gas Pool.
    /// 
    /// Can be triggered automatically by excessive drain or manually by authority.
    /// When paused, no subsidies are distributed until unpaused.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing gas pool and authority
    /// * `reason` - Reason for the pause
    /// 
    /// # Security
    /// 
    /// Only callable by protocol authority.
    pub fn emergency_pause_gas_pool(
        ctx: Context<GasPoolEmergencyControl>,
        reason: EmergencyReason,
    ) -> Result<()> {
        let gas_pool = &mut ctx.accounts.gas_pool_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        require!(
            ctx.accounts.authority.key() == gas_pool.authority,
            ErrorCode::Unauthorized
        );
        
        gas_pool.emergency_paused = true;
        gas_pool.emergency_triggered_at = current_time;
        gas_pool.emergency_reason = reason.clone();
        
        emit!(GasPoolEmergency {
            reason,
            daily_distributed: gas_pool.daily_distributed,
            timestamp: current_time,
        });
        
        Ok(())
    }

    /// Resume Gas Pool operations after emergency.
    /// 
    /// # Security
    /// 
    /// Only callable by protocol authority.
    pub fn resume_gas_pool(ctx: Context<GasPoolEmergencyControl>) -> Result<()> {
        let gas_pool = &mut ctx.accounts.gas_pool_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        require!(
            ctx.accounts.authority.key() == gas_pool.authority,
            ErrorCode::Unauthorized
        );
        
        gas_pool.emergency_paused = false;
        gas_pool.emergency_reason = EmergencyReason::None;
        
        emit!(GasPoolResumed {
            resumed_by: ctx.accounts.authority.key(),
            timestamp: current_time,
        });
        
        Ok(())
    }

    // ========================================================================
    // ANNUAL LIVENESS VERIFICATION
    // ========================================================================

    /// Step 1: Request a challenge for annual liveness verification.
    /// 
    /// This initiates the proof-of-life process. A random challenge is generated
    /// that the user must sign with a live biometric scan within 5 minutes.
    /// 
    /// # Process Overview
    /// 
    /// 1. `request_liveness_challenge()` - Get random challenge (this function)
    /// 2. User reads biometrics from eID card (off-chain, client-side)
    /// 3. User performs LIVE biometric scan (off-chain, client-side)
    /// 4. `verify_annual_liveness()` - Submit proof, system verifies match
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user state
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Challenge generated and stored
    /// * `Err(LivenessVerificationLocked)` - Too many failed attempts, wait 24h
    /// 
    /// # Events
    /// 
    /// Emits `LivenessChallengeIssued` with challenge bytes and expiry time.
    /// 
    /// # Security
    /// 
    /// - Challenge is unique per user, slot, and timestamp
    /// - Expires in 5 minutes (BIOMETRIC_CHALLENGE_TIMEOUT)
    /// - Max 3 failed attempts per day before lockout
    pub fn request_liveness_challenge(ctx: Context<RequestLivenessChallenge>) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;

        // Check if user is locked out from too many failed attempts
        require!(
            current_time >= user.liveness.locked_until,
            ErrorCode::LivenessVerificationLocked
        );

        // Generate random challenge using recent blockhash
        let clock = Clock::get()?;
        let challenge = generate_challenge(
            &ctx.accounts.user.key(),
            clock.slot,
            current_time,
        );

        // Store challenge
        user.liveness.current_challenge = challenge;
        user.liveness.challenge_issued_at = current_time;
        user.liveness.challenge_expires_at = current_time + BIOMETRIC_CHALLENGE_TIMEOUT;

        emit!(LivenessChallengeIssued {
            user: ctx.accounts.user.key(),
            challenge,
            expires_at: user.liveness.challenge_expires_at,
            timestamp: current_time,
        });

        Ok(())
    }

    /// Step 2: Submit biometric proof to complete annual liveness verification.
    /// 
    /// This verifies that the biometrics stored on the user's eID card match
    /// a LIVE biometric scan. A dead person cannot perform a live scan,
    /// preventing dead person fraud without trusting government death registries.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user state
    /// * `proof` - Biometric proof containing eID hash, live scan hash, and attestation
    /// 
    /// # Proof Structure
    /// 
    /// ```ignore
    /// BiometricProof {
    ///     eid_biometric_hash: [u8; 32],      // Hash from eID card
    ///     live_biometric_hash: [u8; 32],     // Hash from LIVE scan
    ///     biometric_type: BiometricType,    // Fingerprint, Iris, or Face
    ///     challenge_response: [u8; 32],     // Signed challenge
    ///     eid_country: [u8; 3],             // ISO country code
    ///     eid_expiry: i64,                  // eID expiration timestamp
    ///     timestamp: i64,                   // Proof generation time
    ///     device_attestation: [u8; 64],     // TEE/Secure Enclave signature
    /// }
    /// ```
    /// 
    /// # Verification Steps
    /// 
    /// 1. User is a verified human
    /// 2. User is not blacklisted
    /// 3. User is not locked out
    /// 4. Challenge exists and hasn't expired
    /// 5. Proof timestamp is within challenge window
    /// 6. eID card hasn't expired
    /// 7. Challenge response is correct
    /// 8. **CRITICAL**: eID biometrics match LIVE biometrics
    /// 9. For returning users: same biometric type as registered
    /// 10. Device attestation is valid
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Verification successful, liveness extended 1 year
    /// * `Err(OnlyHumansCanVerify)` - Not a verified human
    /// * `Err(BiometricMismatch)` - eID and live scan don't match
    /// * `Err(ChallengeExpired)` - 5 minute window passed
    /// * `Err(EidExpired)` - eID card has expired
    /// 
    /// # Events
    /// 
    /// Emits `LivenessVerified` with verification count and next due date.
    /// 
    /// # Security
    /// 
    /// - ALL countries treated equally (any government could be hostile)
    /// - Only individual fraud is punished, no collective penalties
    /// - Device attestation prevents replay attacks
    pub fn verify_annual_liveness(
        ctx: Context<VerifyAnnualLiveness>,
        proof: BiometricProof,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;

        // =====================================================================
        // SECURITY CHECKS
        // =====================================================================

        // 1. Must be a verified human
        require!(user.is_human, ErrorCode::OnlyHumansCanVerify);

        // 2. Must not be blacklisted
        require!(!user.permanently_blacklisted, ErrorCode::UserBlacklisted);

        // 3. Must not be locked out
        require!(
            current_time >= user.liveness.locked_until,
            ErrorCode::LivenessVerificationLocked
        );

        // 4. Challenge must exist and not be expired
        require!(
            user.liveness.challenge_issued_at > 0,
            ErrorCode::NoChallengeIssued
        );
        require!(
            current_time <= user.liveness.challenge_expires_at,
            ErrorCode::ChallengeExpired
        );

        // 5. Proof timestamp must be recent (within challenge window)
        require!(
            proof.timestamp >= user.liveness.challenge_issued_at,
            ErrorCode::ProofTooOld
        );
        require!(
            proof.timestamp <= current_time + 60, // Allow 60s clock drift
            ErrorCode::ProofFromFuture
        );

        // 6. eID must not be expired
        require!(
            proof.eid_expiry > current_time,
            ErrorCode::EidExpired
        );

        // =====================================================================
        // BIOMETRIC VERIFICATION
        // =====================================================================
        // CRITICAL: This is the core proof-of-life check.
        // eID biometrics (from government card) must match LIVE scan.
        // A dead person CANNOT perform a live biometric scan.

        // 7. Verify challenge response is correct
        let expected_response = compute_challenge_response(
            &user.liveness.current_challenge,
            &proof.live_biometric_hash,
            proof.timestamp,
        );
        require!(
            proof.challenge_response == expected_response,
            ErrorCode::InvalidChallengeResponse
        );

        // 8. CRITICAL: Verify eID biometrics match LIVE biometrics
        // This is what proves the person is ALIVE
        require!(
            proof.eid_biometric_hash == proof.live_biometric_hash,
            ErrorCode::BiometricMismatch
        );

        // 9. For returning users: verify it's the SAME person
        if user.liveness.verification_count > 0 {
            // Must use same biometric type as registered
            require!(
                proof.biometric_type == user.liveness.biometric_type,
                ErrorCode::BiometricTypeMismatch
            );
            // Must match stored eID hash (prevents identity theft)
            require!(
                proof.eid_biometric_hash == user.liveness.eid_biometric_hash,
                ErrorCode::IdentityMismatch
            );
        }

        // 10. Verify device attestation (prevents replay attacks)
        require!(
            verify_device_attestation(&proof.device_attestation, &proof),
            ErrorCode::InvalidDeviceAttestation
        );

        // =====================================================================
        // SUCCESS: Update liveness state
        // =====================================================================

        user.liveness.is_verified = true;
        user.liveness.last_verification = current_time;
        user.liveness.next_verification_due = current_time + LIVENESS_INTERVAL;
        user.liveness.verification_count += 1;

        // Store/update biometric identity
        user.liveness.biometric_type = proof.biometric_type;
        user.liveness.eid_biometric_hash = proof.eid_biometric_hash;
        user.liveness.eid_issuing_country = proof.eid_country;
        user.liveness.eid_expiry = proof.eid_expiry;

        // Clear challenge and reset failure counter
        user.liveness.current_challenge = [0u8; 32];
        user.liveness.challenge_issued_at = 0;
        user.liveness.challenge_expires_at = 0;
        user.liveness.failed_attempts = 0;

        emit!(LivenessVerified {
            user: ctx.accounts.user.key(),
            biometric_type: user.liveness.biometric_type.clone(),
            verification_count: user.liveness.verification_count,
            next_due: user.liveness.next_verification_due,
            timestamp: current_time,
        });

        Ok(())
    }

    /// Reports a failed liveness verification attempt.
    /// 
    /// Called when biometric verification fails (e.g., mismatch between eID
    /// and live scan). Tracks failed attempts and locks account after 3 failures.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - Context containing user state
    /// 
    /// # Lockout Rules
    /// 
    /// - Max 3 failed attempts per 24-hour period
    /// - After 3 failures: account locked for 24 hours
    /// - Counter resets after 24 hours without failures
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Failure recorded
    /// 
    /// # Events
    /// 
    /// Emits `LivenessVerificationLockedOut` if max attempts reached.
    /// 
    /// # Note
    /// 
    /// This function also clears the current challenge, requiring a new
    /// challenge request for the next attempt.
    pub fn report_failed_liveness_attempt(
        ctx: Context<ReportFailedLiveness>,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;

        // Reset counter if last attempt was more than 24h ago
        if current_time - user.liveness.last_failed_attempt > VERIFICATION_LOCKOUT {
            user.liveness.failed_attempts = 0;
        }

        user.liveness.failed_attempts += 1;
        user.liveness.last_failed_attempt = current_time;

        // Lock account if too many failures
        if user.liveness.failed_attempts >= MAX_VERIFICATION_ATTEMPTS {
            user.liveness.locked_until = current_time + VERIFICATION_LOCKOUT;

            emit!(LivenessVerificationLockedOut {
                user: ctx.accounts.user.key(),
                failed_attempts: user.liveness.failed_attempts,
                locked_until: user.liveness.locked_until,
                timestamp: current_time,
            });
        }

        // Clear the expired/used challenge
        user.liveness.current_challenge = [0u8; 32];
        user.liveness.challenge_issued_at = 0;
        user.liveness.challenge_expires_at = 0;

        Ok(())
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

// ============================================================================
// LIVENESS VERIFICATION HELPERS
// ============================================================================

/// Determine current liveness status based on timestamps
fn get_liveness_status(liveness: &LivenessState, current_time: i64) -> LivenessStatus {
    // Check if locked out first
    if current_time < liveness.locked_until {
        return LivenessStatus::Locked;
    }

    // First-time users get 30 days to verify (they verified during registration)
    if liveness.verification_count == 0 {
        // New users have implicit verification from registration
        return LivenessStatus::Active;
    }

    let time_until_due = liveness.next_verification_due - current_time;
    let time_since_due = current_time - liveness.next_verification_due;

    if time_until_due > LIVENESS_WARNING_DAYS * SECONDS_PER_DAY {
        // More than 60 days until due - all good
        LivenessStatus::Active
    } else if time_until_due > 0 {
        // Less than 60 days - warning period
        LivenessStatus::Warning
    } else if time_since_due <= LIVENESS_GRACE_PERIOD {
        // Past due but within 30-day grace period
        LivenessStatus::Expired
    } else {
        // Grace period expired - account suspended
        LivenessStatus::Suspended
    }
}

/// Generate a random challenge for liveness verification
/// Uses slot + timestamp + user pubkey for unpredictability
fn generate_challenge(user: &Pubkey, slot: u64, timestamp: i64) -> [u8; 32] {
    use anchor_lang::solana_program::hash::hash;

    let mut data = Vec::with_capacity(48);
    data.extend_from_slice(user.as_ref());
    data.extend_from_slice(&slot.to_le_bytes());
    data.extend_from_slice(&timestamp.to_le_bytes());

    hash(&data).to_bytes()
}

/// Compute expected challenge response
/// The client must sign the challenge with their live biometric
fn compute_challenge_response(
    challenge: &[u8; 32],
    live_biometric_hash: &[u8; 32],
    timestamp: i64,
) -> [u8; 32] {
    use anchor_lang::solana_program::hash::hash;

    let mut data = Vec::with_capacity(72);
    data.extend_from_slice(challenge);
    data.extend_from_slice(live_biometric_hash);
    data.extend_from_slice(&timestamp.to_le_bytes());

    hash(&data).to_bytes()
}

/// Verify device attestation to prevent replay attacks
/// In production, this would verify a signature from a trusted execution environment
fn verify_device_attestation(attestation: &[u8; 64], proof: &BiometricProof) -> bool {
    // TODO: Implement actual device attestation verification
    // This should verify:
    // 1. Attestation comes from a trusted device (TEE/Secure Enclave)
    // 2. Attestation is fresh (tied to this specific proof)
    // 3. Device has not been tampered with

    // For now, just check it's not all zeros (placeholder)
    attestation.iter().any(|&b| b != 0)
}

/// Initialize liveness state for new user
fn initialize_liveness_state(current_time: i64) -> LivenessState {
    LivenessState {
        is_verified: true,  // Initial registration counts as first verification
        last_verification: current_time,
        next_verification_due: current_time + LIVENESS_INTERVAL,
        verification_count: 1,
        biometric_type: BiometricType::None,  // Set during first explicit verification
        eid_biometric_hash: [0u8; 32],
        eid_issuing_country: [0u8; 3],
        eid_expiry: 0,
        failed_attempts: 0,
        last_failed_attempt: 0,
        locked_until: 0,
        current_challenge: [0u8; 32],
        challenge_issued_at: 0,
        challenge_expires_at: 0,
    }
}

// ============================================================================
// MERCHANT HELPER FUNCTIONS
// ============================================================================

/// Resets monthly tracking if 30 days have passed since period start
/// Returns true if reset happened
fn maybe_reset_monthly_period(user: &mut UserState, current_time: i64) -> bool {
    const THIRTY_DAYS: i64 = 30 * SECONDS_PER_DAY;
    
    if current_time - user.monthly_volume_period_start >= THIRTY_DAYS {
        // Reset for new period
        user.monthly_volume_received = 0;
        user.monthly_volume_period_start = current_time;
        user.monthly_unique_payers.clear();
        user.monthly_unique_count = 0;
        return true;
    }
    false
}

/// Updates monthly volume tracking for recipient
fn update_monthly_volume(recipient: &mut UserState, sender: &Pubkey, amount: u64, current_time: i64) {
    // Reset if needed
    maybe_reset_monthly_period(recipient, current_time);
    
    // Add to monthly volume
    recipient.monthly_volume_received += amount as u128;
    
    // Track unique payer for this period
    if !recipient.monthly_unique_payers.contains(sender) {
        recipient.monthly_unique_payers.push(*sender);
        recipient.monthly_unique_count += 1;
    }
}

/// Determines merchant tier using simple OR logic:
/// Qualify via unique_customers >= threshold OR monthly_volume >= threshold
/// 
/// This ensures BOTH types of merchants qualify:
/// - Retail (kavarna, trgovina): many customers, smaller transactions
/// - Service (vodovodar, odvetnik): few customers, large transactions
fn determine_merchant_tier(unique_customers: u32, monthly_volume: u128) -> MerchantTier {
    // Check from highest to lowest tier
    // Enterprise: â‰¥2,000 customers OR â‰¥500,000 AGORA
    if unique_customers >= ENTERPRISE_MIN_CUSTOMERS || monthly_volume >= ENTERPRISE_MIN_VOLUME {
        return MerchantTier::Enterprise;
    }
    
    // Large: â‰¥500 customers OR â‰¥100,000 AGORA
    if unique_customers >= LARGE_MIN_CUSTOMERS || monthly_volume >= LARGE_MIN_VOLUME {
        return MerchantTier::Large;
    }
    
    // Medium: â‰¥100 customers OR â‰¥25,000 AGORA
    if unique_customers >= MEDIUM_MIN_CUSTOMERS || monthly_volume >= MEDIUM_MIN_VOLUME {
        return MerchantTier::Medium;
    }
    
    // Small: â‰¥25 customers OR â‰¥5,000 AGORA
    if unique_customers >= SMALL_MIN_CUSTOMERS || monthly_volume >= SMALL_MIN_VOLUME {
        return MerchantTier::Small;
    }
    
    // Emerging: â‰¥10 customers OR â‰¥1,000 AGORA
    if unique_customers >= EMERGING_MIN_CUSTOMERS || monthly_volume >= EMERGING_MIN_VOLUME {
        return MerchantTier::Emerging;
    }
    
    // Below all thresholds
    MerchantTier::None
}

fn decay_merchant_status(user: &mut UserState) -> Result<()> {
    let previous = user.merchant_status.clone();
    
    user.merchant_status = match user.merchant_status {
        MerchantTier::Enterprise => MerchantTier::Large,
        MerchantTier::Large => MerchantTier::Medium,
        MerchantTier::Medium => MerchantTier::Small,
        MerchantTier::Small => MerchantTier::Emerging,
        MerchantTier::Emerging => MerchantTier::None,
        MerchantTier::None => MerchantTier::None,
    };
    
    if previous != user.merchant_status {
        emit!(MerchantStatusDecayed {
            merchant: user.owner,
            from_tier: previous,
            to_tier: user.merchant_status.clone(),
            timestamp: Clock::get()?.unix_timestamp,
        });
    }
    
    Ok(())
}

fn calculate_merchant_fee(tier: &MerchantTier) -> u64 {
    match tier {
        MerchantTier::None => BASE_FEE_RATE,
        MerchantTier::Emerging => BASE_FEE_RATE * 75 / 100,  // 25% off
        MerchantTier::Small => BASE_FEE_RATE * 50 / 100,     // 50% off
        MerchantTier::Medium => BASE_FEE_RATE * 25 / 100,    // 75% off
        MerchantTier::Large | MerchantTier::Enterprise => 0,  // FREE
    }
}

/// Calculates transfer fee based on user's activity and merchant status
/// 
/// Fee hierarchy (lowest fee wins):
/// 1. Merchant status (if any) - based on tier
/// 2. Activity-based multiplier - rewards active users
/// 
/// This encourages:
/// - Using AGORA as currency (active users pay less)
/// - Building merchant economy (merchants pay least)
/// - Discourages hoarding (dormant users pay more)
fn calculate_transfer_fee(user: &UserState, amount: u64, current_time: i64) -> u64 {
    // Merchants use merchant fee (usually lower)
    if user.merchant_status != MerchantTier::None {
        let merchant_rate = calculate_merchant_fee(&user.merchant_status);
        return (amount * merchant_rate) / FEE_DIVISOR;
    }
    
    // Regular users: activity-based fee
    let days_since_last_tx = (current_time - user.last_tx_timestamp) / SECONDS_PER_DAY;
    
    let activity_multiplier = if days_since_last_tx <= ACTIVE_USER_DAYS {
        ACTIVE_FEE_MULTIPLIER      // 0.8x - reward for being active
    } else if days_since_last_tx <= NORMAL_USER_DAYS {
        NORMAL_FEE_MULTIPLIER      // 1.0x - standard rate
    } else if days_since_last_tx <= INACTIVE_USER_DAYS {
        INACTIVE_FEE_MULTIPLIER    // 1.5x - slight penalty
    } else {
        DORMANT_FEE_MULTIPLIER     // 2.0x - dormant penalty
    };
    
    // Calculate fee: amount * base_rate * multiplier / 100 / FEE_DIVISOR
    // Reordered to avoid overflow: (amount * base_rate * multiplier) / (100 * FEE_DIVISOR)
    (amount * BASE_FEE_RATE * activity_multiplier) / (100 * FEE_DIVISOR)
}

/// Returns activity status string for events
fn get_activity_status(last_tx_timestamp: i64, current_time: i64) -> String {
    let days_since_last_tx = (current_time - last_tx_timestamp) / SECONDS_PER_DAY;
    
    if days_since_last_tx <= ACTIVE_USER_DAYS {
        "active".to_string()
    } else if days_since_last_tx <= NORMAL_USER_DAYS {
        "normal".to_string()
    } else if days_since_last_tx <= INACTIVE_USER_DAYS {
        "inactive".to_string()
    } else {
        "dormant".to_string()
    }
}

fn determine_claim_interval(user: &UserState) -> i64 {
    // Based on contribution status
    if user.merchant_status >= MerchantTier::Small 
        || user.sponsor_tier >= SponsorTier::Bronze 
        || user.dao_tier >= DaoTier::Contributor {
        SECONDS_PER_DAY  // Daily claims
    } else {
        SECONDS_PER_DAY * 30  // Monthly for non-contributors
    }
}

fn calculate_retroactive_claim(age_in_days: u64) -> u64 {
    if age_in_days >= MAX_RETROACTIVE_DAYS {
        MAX_RETROACTIVE_DAYS * DAILY_AMOUNT
    } else {
        age_in_days * DAILY_AMOUNT
    }
}

fn calculate_claimable_amount(user: &UserState, current_time: i64) -> Result<u64> {
    let seconds_elapsed = current_time - user.last_claim_timestamp;
    let days_elapsed = (seconds_elapsed / SECONDS_PER_DAY) as u64;
    let claimable_days = std::cmp::min(days_elapsed, MAX_ACCUMULATION_DAYS);
    Ok(claimable_days * DAILY_AMOUNT)
}

// ============================================================================
// CIVIC PASS INTEGRATION
// ============================================================================
// 
// STATUS: PLACEHOLDER - Production implementation required before mainnet
// 
// CIVIC PASS OVERVIEW:
// Civic Pass is a tokenized identity verification system built on the Gateway
// Protocol. It enables users to prove aspects of their identity to smart
// contracts without exposing personal data on-chain.
//
// Key Properties:
// - Non-transferable attestation token (soulbound)
// - Stores cryptographic proofs, not personal data
// - Supports multiple verification levels (CAPTCHA, Liveness, ID, KYC)
// - Over 2M verifications processed on Solana
//
// INTEGRATION ARCHITECTURE:
// â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
// â”‚                        AGORA REGISTRATION FLOW                      â”‚
// â”œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¤
// â”‚  1. User connects wallet (Phantom, Solflare, etc.)                  â”‚
// â”‚  2. Frontend checks for existing Civic Pass via Gateway Protocol    â”‚
// â”‚  3. If no pass: redirect to Civic verification (ID + Liveness)      â”‚
// â”‚  4. Civic issues Gateway Token (on-chain, non-transferable)         â”‚
// â”‚  5. User calls register_user() with gateway_token account           â”‚
// â”‚  6. Smart contract verifies gateway token via CPI                   â”‚
// â”‚  7. If valid + not expired: proceed with registration               â”‚
// â”‚  8. Biometric hash stored for deduplication (separate from Civic)   â”‚
// â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
//
// REQUIRED DEPENDENCIES (Cargo.toml):
// ```toml
// [dependencies]
// solana-gateway = "0.4"  # Gateway Protocol for on-chain verification
// # OR use anchor-gateway for Anchor integration
// ```
//
// REQUIRED ACCOUNTS (RegisterUser context):
// ```rust
// /// The user's Civic Pass (Gateway Token)
// /// Must be owned by Gateway Program and associated with user's wallet
// #[account(
//     constraint = gateway_token.owner_wallet == user.key(),
//     constraint = gateway_token.gatekeeper_network == CIVIC_GATEKEEPER_NETWORK,
// )]
// pub gateway_token: Account<'info, GatewayToken>,
// 
// /// Gateway Program for CPI verification
// pub gateway_program: Program<'info, Gateway>,
// ```
//
// GATEKEEPER NETWORKS (Pass Types):
// - ignREusXmGrscGNUesoU9mxfds9AiYTezUKex2PsZV6  : CAPTCHA (bot resistance)
// - tgnuXXNMDLK8dy7Xm1TdeGyc95MDym4bvAQCwcW21Bf  : Uniqueness (Sybil resistance)  
// - bni1ewus6aMxTxBi5SAfzEmmXLf8KcVFRmTfproJuKw  : ID Verification
// - gatbGF9DvLAw3kWyn1EmH5Nh1Sqp8sTukF7yaQpSc71  : ID + Liveness (RECOMMENDED)
//
// AGORA RECOMMENDATION: Use "ID + Liveness" (gatbGF9DvLAw...) for:
// - Government ID verification (passport, national ID)
// - Live selfie matching (proves current possession)
// - One person = one pass guarantee
//
// IMPLEMENTATION STEPS:
//
// STEP 1: Add Gateway Token to RegisterUser accounts
// ```rust
// use solana_gateway::state::GatewayToken;
// use solana_gateway::Gateway;
// 
// pub struct RegisterUser<'info> {
//     // ... existing accounts ...
//     
//     /// Gateway token (Civic Pass) - must be valid and not expired
//     #[account(
//         constraint = gateway_token.owner_wallet == user.key() 
//             @ ErrorCode::GatewayTokenOwnerMismatch,
//         constraint = gateway_token.gatekeeper_network == protocol_state.civic_gatekeeper_network 
//             @ ErrorCode::InvalidGatekeeperNetwork,
//     )]
//     pub gateway_token: Account<'info, GatewayToken>,
//     
//     /// CHECK: Gateway program for verification CPI
//     #[account(address = solana_gateway::id())]
//     pub gateway_program: UncheckedAccount<'info>,
// }
// ```
//
// STEP 2: Implement verification function
// ```rust
// fn verify_civic_pass_production(
//     gateway_token: &Account<GatewayToken>,
//     user: &Pubkey,
//     gatekeeper_network: &Pubkey,
//     current_time: i64,
// ) -> Result<()> {
//     // 1. Verify owner matches
//     require!(
//         gateway_token.owner_wallet == *user,
//         ErrorCode::GatewayTokenOwnerMismatch
//     );
//     
//     // 2. Verify correct gatekeeper network (ID + Liveness)
//     require!(
//         gateway_token.gatekeeper_network == *gatekeeper_network,
//         ErrorCode::InvalidGatekeeperNetwork
//     );
//     
//     // 3. Verify token is active (not revoked)
//     require!(
//         gateway_token.state == GatewayTokenState::Active,
//         ErrorCode::GatewayTokenNotActive
//     );
//     
//     // 4. Verify not expired
//     if let Some(expiry) = gateway_token.expire_time {
//         require!(
//             current_time < expiry,
//             ErrorCode::GatewayTokenExpired
//         );
//     }
//     
//     Ok(())
// }
// ```
//
// STEP 3: Add to ProtocolState
// ```rust
// pub struct ProtocolState {
//     // ... existing fields ...
//     pub civic_gatekeeper_network: Pubkey,  // Set during initialize()
// }
// ```
//
// STEP 4: Frontend integration (@civic/solana-gateway-react)
// ```typescript
// import { GatewayProvider, useGateway } from "@civic/solana-gateway-react";
// import { findGatewayToken } from "@identity.com/solana-gateway-ts";
// 
// // Wrap app in GatewayProvider
// <GatewayProvider
//   wallet={wallet}
//   gatekeeperNetwork={new PublicKey("gatbGF9DvLAw...")}
//   connection={connection}
// >
//   <App />
// </GatewayProvider>
// 
// // In registration component
// const { gatewayToken, requestGatewayToken } = useGateway();
// 
// if (!gatewayToken) {
//   return <button onClick={requestGatewayToken}>Verify Identity</button>;
// }
// 
// // User has pass, can register
// await program.methods
//   .registerUser(ageInDays, [], biometricHash)
//   .accounts({
//     gatewayToken: gatewayToken.publicKey,
//     // ... other accounts
//   })
//   .rpc();
// ```
//
// ERROR CODES TO ADD:
// ```rust
// #[error_code]
// pub enum ErrorCode {
//     #[msg("Gateway token owner does not match user")]
//     GatewayTokenOwnerMismatch,
//     
//     #[msg("Invalid gatekeeper network - must use approved Civic Pass type")]
//     InvalidGatekeeperNetwork,
//     
//     #[msg("Gateway token is not active (may be revoked)")]
//     GatewayTokenNotActive,
//     
//     #[msg("Gateway token has expired - please renew your Civic Pass")]
//     GatewayTokenExpired,
// }
// ```
//
// TESTING ON DEVNET:
// 1. Get test Civic Pass: https://civic.me (select devnet)
// 2. Use devnet gatekeeper network for testing
// 3. Verify token exists: findGatewayToken(connection, wallet, network)
//
// SECURITY CONSIDERATIONS:
// - Gateway tokens are non-transferable (soulbound)
// - Civic stores personal data OFF-chain (privacy preserving)
// - On-chain: only cryptographic attestation stored
// - Expiry times ensure passes remain valid
// - Revocation possible if fraud detected
//
// DUAL VERIFICATION MODEL (AGORA-specific):
// AGORA uses BOTH Civic Pass AND biometric hash for maximum security:
// 
// Layer 1: Civic Pass (Government ID + Liveness)
//   - Proves: Real identity document exists
//   - Proves: Person is alive (selfie matches ID)
//   - Weakness: Same person could use multiple wallets
//
// Layer 2: Biometric Hash (Fingerprint deduplication)
//   - Proves: Same fingerprints = same person
//   - Prevents: One person registering multiple times
//   - Weakness: Requires biometric hardware
//
// Combined: Maximum Sybil resistance
//   - Must have valid government ID (Civic)
//   - Must have unique fingerprints (Biometric)
//   - Must be alive (Liveness check)
//
// ============================================================================

/// Verifies that the user has a valid Civic Pass (Gateway Token).
/// 
/// # Current Status
/// 
/// **PLACEHOLDER IMPLEMENTATION** - Returns Ok if proof is non-empty.
/// Production implementation requires Gateway Protocol integration.
/// 
/// # Production Implementation
/// 
/// Will verify:
/// 1. Gateway token owner matches user wallet
/// 2. Gatekeeper network is AGORA-approved (ID + Liveness)
/// 3. Token state is Active (not revoked)
/// 4. Token has not expired
/// 
/// # Arguments
/// 
/// * `proof` - Placeholder for civic pass proof bytes
///             In production: replaced by GatewayToken account
/// 
/// # Returns
/// 
/// * `Ok(())` - Verification passed (placeholder: proof non-empty)
/// * `Err(InvalidCivicPass)` - Verification failed
/// 
/// # Security Note
/// 
/// This placeholder MUST be replaced before mainnet deployment.
/// Current implementation provides NO actual identity verification.
/// 
/// # See Also
/// 
/// - Civic Pass Docs: https://docs.civic.com
/// - Gateway Protocol: https://github.com/identity-com/on-chain-identity-gateway
/// - Integration Guide: technical-architecture.md Section 4
fn verify_civic_pass(proof: &[u8]) -> Result<()> {
    // =========================================================================
    // PLACEHOLDER IMPLEMENTATION
    // =========================================================================
    // 
    // WARNING: This is NOT production-ready!
    // Only checks that proof bytes are non-empty.
    // 
    // Before Mainnet:
    // 1. Add solana-gateway dependency
    // 2. Add GatewayToken to RegisterUser accounts
    // 3. Implement verify_civic_pass_production() as shown above
    // 4. Remove this placeholder
    //
    // Estimated implementation time: 2-3 days
    // Dependencies: solana-gateway = "0.4"
    // =========================================================================
    
    require!(proof.len() > 0, ErrorCode::InvalidCivicPass);
    
    // Log warning in devnet/testnet
    msg!("WARNING: Using placeholder Civic Pass verification");
    msg!("Production deployment requires Gateway Protocol integration");
    
    Ok(())
}

// ============================================================================
// GAS POOL HELPER FUNCTIONS
// ============================================================================

/// Calculate sponsor tier based on total contribution.
fn calculate_sponsor_tier(total_contribution: u64) -> SponsorTier {
    if total_contribution >= DIAMOND_THRESHOLD {
        SponsorTier::Diamond
    } else if total_contribution >= PLATINUM_THRESHOLD {
        SponsorTier::Platinum
    } else if total_contribution >= GOLD_THRESHOLD {
        SponsorTier::Gold
    } else if total_contribution >= SILVER_THRESHOLD {
        SponsorTier::Silver
    } else if total_contribution >= BRONZE_THRESHOLD {
        SponsorTier::Bronze
    } else {
        SponsorTier::None
    }
}

/// Get personal allocation percentage for a tier.
fn get_personal_allocation_pct(tier: &SponsorTier) -> u64 {
    match tier {
        SponsorTier::None => 0,
        SponsorTier::Bronze => BRONZE_PERSONAL_PCT,
        SponsorTier::Silver => SILVER_PERSONAL_PCT,
        SponsorTier::Gold => GOLD_PERSONAL_PCT,
        SponsorTier::Platinum => PLATINUM_PERSONAL_PCT,
        SponsorTier::Diamond => DIAMOND_PERSONAL_PCT,
    }
}

/// Get fee discount percentage for a sponsor tier.
pub fn get_sponsor_fee_discount(tier: &SponsorTier) -> u64 {
    match tier {
        SponsorTier::None => 0,
        SponsorTier::Bronze => BRONZE_FEE_DISCOUNT,
        SponsorTier::Silver => SILVER_FEE_DISCOUNT,
        SponsorTier::Gold => GOLD_FEE_DISCOUNT,
        SponsorTier::Platinum => PLATINUM_FEE_DISCOUNT,
        SponsorTier::Diamond => DIAMOND_FEE_DISCOUNT,
    }
}

/// Estimate gas cost for a transaction type.
fn estimate_gas_cost(tx_type: &TransactionType) -> u64 {
    // Approximate costs in lamports
    match tx_type {
        TransactionType::Claim => 5_000,          // Simple token mint
        TransactionType::Transfer => 5_000,        // Token transfer
        TransactionType::Register => 10_000,       // Account creation
        TransactionType::BusinessTx => 7_000,      // Slightly more complex
    }
}

/// Apply free tier limits and return subsidy amount if allowed.
fn apply_free_tier_limits(
    daily_count: u8,
    tracker: &TransactionTracker,
    current_time: i64,
) -> Result<u64> {
    // Check daily limit
    require!(
        daily_count < FREE_TIER_DAILY_TX,
        ErrorCode::DailySubsidyLimitExceeded
    );
    
    // Check cooldown (last TX must be > 60 seconds ago)
    let last_tx_time = tracker.recent_timestamps[(tracker.buffer_index.saturating_sub(1) % 10) as usize];
    if last_tx_time > 0 {
        require!(
            current_time - last_tx_time >= FREE_TIER_COOLDOWN,
            ErrorCode::SubsidyCooldownActive
        );
    }
    
    // Return standard subsidy amount
    Ok(5_000) // ~0.000005 SOL per TX
}

/// Detect ping-pong attack pattern (Aâ†’Bâ†’A).
fn detect_ping_pong(
    tracker: &TransactionTracker,
    sender: &Pubkey,
    recipient: &Pubkey,
    current_time: i64,
) -> bool {
    // Look for: we sent to this recipient before, and now they're sending back
    for i in 0..10 {
        let idx = i as usize;
        let prev_recipient = tracker.recent_recipients[idx];
        let prev_time = tracker.recent_timestamps[idx];
        
        // Skip empty slots
        if prev_time == 0 {
            continue;
        }
        
        // Check if within ping-pong window
        if current_time - prev_time > PING_PONG_WINDOW {
            continue;
        }
        
        // Check if this is a return transaction
        // If we previously sent to X, and now X is the sender (recipient of our TX)
        // AND the current recipient is us... that's ping-pong
        // Actually: if prev_recipient == sender, means we sent TO them before
        // Now they're sending back TO us (recipient == our address)
        if prev_recipient == *sender {
            // We sent to this person before, now they're sending (back)
            // This is suspicious but not necessarily ping-pong
            // True ping-pong: A sends to B, B sends back to A
            // So: if recipient == tracker.user (the one whose tracker this is)
            // But we don't have tracker.user in this context...
            // 
            // Simplified: if we see the same two addresses in quick succession
            // with reversed direction, it's suspicious
            return true;
        }
    }
    
    false
}

/// Count transactions to same recipient in time window.
fn count_same_recipient(
    tracker: &TransactionTracker,
    recipient: &Pubkey,
    current_time: i64,
    window_seconds: i64,
) -> u8 {
    let mut count = 0u8;
    
    for i in 0..10 {
        let idx = i as usize;
        let prev_recipient = tracker.recent_recipients[idx];
        let prev_time = tracker.recent_timestamps[idx];
        
        if prev_time == 0 {
            continue;
        }
        
        if current_time - prev_time > window_seconds {
            continue;
        }
        
        if prev_recipient == *recipient {
            count += 1;
        }
    }
    
    count
}

/// Transaction type for gas estimation.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum TransactionType {
    /// Daily UBI claim
    Claim,
    /// AGORA token transfer
    Transfer,
    /// User registration
    Register,
    /// Business transaction
    BusinessTx,
}

fn handle_fees<'info>(
    ctx: &Context<'_, '_, '_, 'info, BusinessTransaction<'info>>,
    fee_amount: u64,
) -> Result<()> {
    // Burn 50%
    let burn_amount = (fee_amount * BURN_PERCENTAGE) / 100;
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.from.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        burn_amount,
    )?;
    
    // Treasury 50%
    let treasury_amount = fee_amount - burn_amount;
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.treasury_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        treasury_amount,
    )?;
    
    Ok(())
}

// ============================================================================
// ACCOUNT STRUCTURES
// ============================================================================

/// Global protocol state - stores configuration and statistics.
/// 
/// This is a Program Derived Address (PDA) seeded by `["protocol"]`.
/// There is exactly one ProtocolState per deployment.
/// 
/// # Fields
/// 
/// - `authority`: Account that can perform admin functions (transfers to DAO)
/// - `mint`: The AGORA token mint address
/// - `treasury`: Receives 50% of transaction fees
/// - `gas_pool`: Holds SOL for gas subsidies to new users
/// - `total_users`: Count of registered humans
/// - `total_merchants`: Count of users with merchant status
/// - `total_businesses`: Count of registered businesses
/// - `total_minted`: Lifetime AGORA tokens minted (retroactive + daily claims)
/// - `total_burned`: Lifetime AGORA tokens burned (from fees)
/// - `emergency_paused`: If true, protocol is in emergency mode
/// - `business_registration_frozen`: If true, new businesses cannot register
#[account]
pub struct ProtocolState {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub treasury: Pubkey,
    pub gas_pool: Pubkey,
    pub biometric_registry: Pubkey,          // Global biometric hash registry
    pub total_users: u64,
    pub total_merchants: u64,
    pub total_businesses: u64,
    pub total_minted: u128,
    pub total_burned: u128,
    pub launch_timestamp: i64,
    pub emergency_paused: bool,
    pub business_registration_frozen: bool,  // Emergency control
    pub paused_at: i64,
    pub bump: u8,
}

// ============================================================================
// GAS POOL STATE
// ============================================================================
// Manages SOL subsidies for user transaction fees.
// PDA-controlled (no human gatekeepers) - fully trustless.
//
// Architecture:
// - Sponsors deposit SOL â†’ receive tier benefits
// - Pool subsidizes gas for new users
// - Anti-abuse system prevents drain attacks
// - Emergency brake pauses if drain rate too high

/// Gas Pool state - manages SOL subsidies for transaction fees.
/// 
/// This is a Program Derived Address (PDA) seeded by `["gas_pool"]`.
/// 
/// # Purpose
/// 
/// Every Solana transaction requires SOL for gas fees (~0.00025 SOL).
/// This creates a barrier for new users. The Gas Pool subsidizes these
/// fees so users can claim and transfer AGORA without needing SOL.
/// 
/// # Funding Model
/// 
/// Sponsors deposit SOL and receive:
/// - Personal allocation (portion for their own gas)
/// - Fee discounts on AGORA transfers
/// - Priority transaction access
/// - DAO governance weight (future)
/// 
/// # Security
/// 
/// Multiple anti-abuse mechanisms:
/// - Rate limiting (daily TX limits)
/// - Ping-pong detection (Aâ†’Bâ†’A patterns)
/// - Minimum transfer amounts for subsidy
/// - Emergency brake if drain rate too high
#[account]
pub struct GasPoolState {
    /// PDA authority for the pool
    pub authority: Pubkey,
    
    /// Total SOL deposited by all sponsors (lifetime)
    pub total_deposited: u64,
    
    /// Total SOL distributed as gas subsidies (lifetime)
    pub total_distributed: u64,
    
    /// Current available balance in pool
    pub available_balance: u64,
    
    /// Amount distributed in current 24h period
    pub daily_distributed: u64,
    
    /// Timestamp when daily counter resets
    pub daily_reset_time: i64,
    
    /// Number of sponsors (all tiers)
    pub total_sponsors: u64,
    
    /// Is pool in emergency pause mode?
    pub emergency_paused: bool,
    
    /// When emergency was triggered
    pub emergency_triggered_at: i64,
    
    /// Reason for emergency (if any)
    pub emergency_reason: EmergencyReason,
    
    /// PDA bump seed
    pub bump: u8,
}

/// Individual sponsor record - tracks a sponsor's contribution and usage.
/// 
/// PDA seeded by `["sponsor", user_pubkey]`.
/// 
/// # Tiered System
/// 
/// | Tier | Contribution | Personal Use | Fee Discount |
/// |------|-------------|--------------|--------------|
/// | Bronze | 1 SOL | 20% | 25% |
/// | Silver | 10 SOL | 15% | 50% |
/// | Gold | 100 SOL | 10% | 75% |
/// | Platinum | 1,000 SOL | 5% | 100% |
/// | Diamond | 10,000 SOL | 3% | 100% |
/// 
/// # Personal Allocation
/// 
/// Sponsors receive a portion of their contribution for personal gas use.
/// When depleted, they drop to free tier limits until they contribute more.
#[account]
pub struct SponsorRecord {
    /// Sponsor's wallet address
    pub sponsor: Pubkey,
    
    /// Current sponsor tier
    pub tier: SponsorTier,
    
    /// Total SOL contributed (lifetime)
    pub total_contributed: u64,
    
    /// Personal allocation remaining (in lamports)
    pub personal_allocation: u64,
    
    /// Amount added to community pool
    pub pool_contribution: u64,
    
    /// When they first became a sponsor
    pub sponsored_at: i64,
    
    /// When tier was last upgraded
    pub tier_upgraded_at: i64,
    
    /// Subsidized TX count today
    pub daily_subsidy_count: u8,
    
    /// When daily counter resets
    pub daily_reset_time: i64,
    
    /// PDA bump seed
    pub bump: u8,
}

/// Tracks recent transactions for anti-abuse detection.
/// 
/// PDA seeded by `["tx_tracker", user_pubkey]`.
/// 
/// Used to detect:
/// - Ping-pong attacks (Aâ†’Bâ†’A rapid cycles)
/// - Rapid fire attacks (many TX in short time)
/// - Same-recipient abuse (many TX to one address)
#[account]
pub struct TransactionTracker {
    /// User this tracker belongs to
    pub user: Pubkey,
    
    /// Last 10 transaction recipients (circular buffer)
    pub recent_recipients: [Pubkey; 10],
    
    /// Timestamps of last 10 transactions
    pub recent_timestamps: [i64; 10],
    
    /// Current index in circular buffer
    pub buffer_index: u8,
    
    /// Transactions in last minute
    pub tx_last_minute: u8,
    
    /// Last minute timestamp
    pub minute_start: i64,
    
    /// Suspicious activity score (0-100)
    pub suspicion_score: u8,
    
    /// PDA bump seed
    pub bump: u8,
}

/// Reason for emergency pause
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Default)]
pub enum EmergencyReason {
    #[default]
    None,
    /// Daily drain exceeded threshold
    ExcessiveDrain,
    /// Detected attack pattern
    AttackDetected,
    /// Pool balance critically low
    LowBalance,
    /// Manual pause by authority
    ManualPause,
}

// ============================================================================
// BIOMETRIC DEDUPLICATION REGISTRY
// ============================================================================
// Global registry of all biometric hashes - prevents duplicate accounts
// hash(left_index_finger + right_index_finger) = unique human identifier
// eIDAS mandates EXACTLY both index fingers - cannot use other fingers

#[account]
pub struct BiometricRegistry {
    pub total_registered: u64,               // Total unique humans registered
    pub duplicate_attempts: u64,             // Count of rejected duplicates
    pub last_updated: i64,
    pub bump: u8,
}

// Individual biometric entry (PDA per hash)
// If someone tries to register with same fingerprints, PDA init fails = duplicate blocked
#[account]
pub struct BiometricEntry {
    pub biometric_hash: [u8; 32],            // SHA-256(left_index + right_index)
    pub user_account: Pubkey,                // Associated user account
    pub registered_at: i64,
    pub is_active: bool,                     // Can be deactivated if individual fraud proven
    pub bump: u8,
}

// ============================================================================
// COUNTRY SANCTIONS
// ============================================================================
// DAO can vote to impose sanctions on countries committing atrocities.
// Sanctions reduce UBI for citizens of that country temporarily.
// PDA seeded by ["country_sanction", country_code]

/// Country sanction record - tracks DAO-imposed sanctions on nations.
/// 
/// # Philosophy
/// 
/// When a government commits genocide, war crimes, or mass atrocities,
/// the international community should respond. Traditional sanctions hurt
/// citizens while leaders remain comfortable. AGORA sanctions are different:
/// 
/// - Citizens still receive UBI (reduced, not zero) - they are victims too
/// - Sanctions are temporary - pressure for change, not permanent punishment
/// - DAO votes required - democratic decision, not unilateral
/// - Transparent on-chain - everyone can see why and for how long
/// 
/// # Fields
/// 
/// - `country_code`: ISO 3166-1 alpha-3 (e.g., "ISR", "RUS", "USA")
/// - `sanction_rate`: Percentage of normal UBI (1000 = 10%, 5000 = 50%)
/// - `reason`: Brief description of the atrocity
/// - `evidence_hash`: IPFS hash of evidence documentation
/// - `start_timestamp`: When sanction began
/// - `end_timestamp`: When sanction expires
/// - `dao_proposal_id`: Reference to the DAO vote that imposed this
/// 
/// # Example
/// 
/// Country commits genocide → DAO votes → Citizens receive 10% UBI for 4 months
/// → Government faces economic pressure → Sanction can be renewed or lifted
#[account]
pub struct CountrySanction {
    pub country_code: [u8; 3],              // ISO 3166-1 alpha-3
    pub is_active: bool,
    pub sanction_rate: u64,                 // % of normal UBI (1000 = 10%)
    pub reason: String,                     // Brief description (max 256 chars)
    pub evidence_hash: [u8; 32],            // IPFS hash of evidence
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub dao_proposal_id: u64,               // Which DAO proposal imposed this
    pub imposed_by: Pubkey,                 // DAO authority that executed
    pub vote_count_for: u64,                // How many voted for sanction
    pub vote_count_against: u64,            // How many voted against
    pub bump: u8,
}

/// Registry of all country sanctions
#[account]
pub struct SanctionRegistry {
    pub total_active_sanctions: u32,
    pub total_historical_sanctions: u32,
    pub last_updated: i64,
    pub bump: u8,
}

// ============================================================================
// DAO PROPOSAL SYSTEM
// ============================================================================

/// Proposal types determine bond amount, voting period, and approval threshold
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ProposalType {
    Standard,       // General governance (100 AGORA bond)
    Treasury,       // Spending treasury funds (500 AGORA bond)
    Constitutional, // Protocol parameter changes (1000 AGORA bond)
    Sanction,       // Country sanctions (2000 AGORA bond)
}

/// Proposal status lifecycle
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,         // Voting in progress
    Passed,         // Reached quorum + approval threshold
    Rejected,       // Reached quorum but below approval threshold
    Expired,        // Voting period ended without quorum
    Executed,       // Passed and executed
    Cancelled,      // Cancelled by proposer (bond forfeited)
}

/// DAO Proposal - represents a governance proposal with bond mechanism
/// 
/// # Bond Mechanism (Anti-Spam)
/// 
/// Proposer must lock AGORA tokens as bond. Bond is returned if:
/// - Proposal reaches >= 50% of quorum (even if rejected)
/// 
/// Bond is forfeited to treasury if:
/// - Proposal fails to reach 50% of quorum (spam/irrelevant)
/// - Proposer cancels the proposal
/// 
/// This prevents flooding DAO with garbage proposals while still
/// allowing controversial proposals that generate discussion.
#[account]
pub struct Proposal {
    /// Unique proposal ID (incrementing)
    pub id: u64,
    
    /// Proposer's public key
    pub proposer: Pubkey,
    
    /// Proposal type (determines parameters)
    pub proposal_type: ProposalType,
    
    /// Current status
    pub status: ProposalStatus,
    
    /// Title (max 64 chars, stored as bytes)
    pub title: [u8; 64],
    
    /// Description IPFS hash (for full proposal text)
    pub description_hash: [u8; 32],
    
    /// Bond amount locked (in AGORA base units)
    pub bond_amount: u64,
    
    /// Whether bond has been returned/forfeited
    pub bond_resolved: bool,
    
    /// Voting
    pub votes_yes: u64,
    pub votes_no: u64,
    pub votes_abstain: u64,
    pub total_voters: u64,
    
    /// Required quorum for this proposal type
    pub quorum_required: u64,
    
    /// Approval threshold (basis points)
    pub approval_threshold: u64,
    
    /// Timestamps
    pub created_at: i64,
    pub voting_ends_at: i64,
    pub executed_at: i64,
    
    /// For treasury proposals: amount to transfer
    pub treasury_amount: u64,
    
    /// For treasury proposals: recipient
    pub treasury_recipient: Pubkey,
    
    /// For sanction proposals: country code
    pub sanction_country: [u8; 3],
    
    /// PDA bump
    pub bump: u8,
}

/// Tracks which proposals a user has voted on (prevents double voting)
#[account]
pub struct VoteRecord {
    /// User who voted
    pub voter: Pubkey,
    
    /// Proposal ID
    pub proposal_id: u64,
    
    /// Vote choice (0 = no, 1 = yes, 2 = abstain)
    pub choice: u8,
    
    /// Timestamp of vote
    pub voted_at: i64,
    
    /// PDA bump
    pub bump: u8,
}

/// Global proposal counter and stats
#[account]
pub struct ProposalRegistry {
    /// Next proposal ID to assign
    pub next_proposal_id: u64,
    
    /// Total proposals created
    pub total_proposals: u64,
    
    /// Active proposals count
    pub active_proposals: u32,
    
    /// Total bonds collected (forfeited)
    pub total_bonds_forfeited: u64,
    
    /// Total bonds returned
    pub total_bonds_returned: u64,
    
    /// PDA bump
    pub bump: u8,
}

/// User account state - stores all information for a registered human.
/// 
/// This is a Program Derived Address (PDA) seeded by `["user", user_pubkey]`.
/// Each verified human has exactly one UserState account.
/// 
/// # Identity & Verification
/// 
/// - `owner`: User's wallet public key
/// - `civic_pass`: Civic Pass verification reference
/// - `is_human`: Must be true for UBI claims (businesses can't claim)
/// - `biometric_hash`: SHA-256 of fingerprints for deduplication
/// 
/// # UBI Claims
/// 
/// - `registration_timestamp`: When user registered
/// - `last_claim_timestamp`: When user last claimed daily UBI
/// - `age_in_days_at_registration`: Used for retroactive claim calculation
/// - `total_claimed`: Lifetime AGORA claimed
/// 
/// # Merchant Detection (30-day rolling metrics)
/// 
/// - `merchant_status`: Current tier (None â†’ Enterprise)
/// - `monthly_volume_received`: Volume in current 30-day period
/// - `monthly_unique_count`: Unique payers in current period
/// 
/// # Activity Tracking (for fee calculation)
/// 
/// - `last_tx_timestamp`: Last transaction time (determines activity multiplier)
/// - `volume_sent/received`: Lifetime statistics
/// 
/// # Business
/// 
/// - `business_profile`: Optional single business (None or Some)
/// 
/// # Security
/// 
/// - `violations`: Count of rule violations (2 = permanent ban)
/// - `suspended_until`: Timestamp when suspension ends
/// - `permanently_blacklisted`: If true, account is permanently banned
/// - `liveness`: Annual liveness verification state
#[account]
pub struct UserState {
    pub owner: Pubkey,
    pub civic_pass: Pubkey,
    pub is_human: bool,                    // CRITICAL: Must be true for claims
    pub registration_timestamp: i64,
    pub last_claim_timestamp: i64,
    pub last_tx_timestamp: i64,
    pub last_merchant_tx: i64,
    pub age_in_days_at_registration: u64,
    pub is_child: bool,
    pub total_claimed: u128,

    // =========================================================================
    // CHILD PROTECTION - Two Pool System
    // =========================================================================
    // Children have funds split into accessible (can spend) and locked (until 18).
    // Accessible rate: 1/(π×e) ≈ 11.69% under 16, φ/(π×e) ≈ 18.92% ages 16-18.
    // At age 18: locked_balance transfers to accessible_balance.
    // Adults (18+): All funds go directly to accessible_balance.
    
    pub accessible_balance: u64,            // Can be spent freely
    pub locked_balance: u64,                // Locked until age 18
    pub birth_timestamp: i64,               // For calculating current age

    // Biometric identity - for deduplication only
    pub biometric_hash: [u8; 32],           // SHA-256(left_index + right_index)

    // Merchant fields - Simple OR Logic
    // Qualify via: unique_customers >= threshold OR monthly_volume >= threshold
    pub merchant_status: MerchantTier,
    pub volume_received: u128,             // Total AGORA received (lifetime)
    pub volume_sent: u128,                 // Total AGORA sent (lifetime)
    pub tx_count_received: u64,            // Number of incoming TX
    pub tx_count_sent: u64,                // Number of outgoing TX
    pub unique_payers: Vec<Pubkey>,        // Who paid this merchant
    pub unique_payers_count: u32,          // Count for tier calculation
    
    // 30-day rolling volume tracking for merchant detection
    pub monthly_volume_received: u128,     // Volume in current 30-day period
    pub monthly_volume_period_start: i64,  // When current period started
    pub monthly_unique_payers: Vec<Pubkey>, // Unique payers in current period
    pub monthly_unique_count: u32,         // Count for current period
    
    // SINGLE BUSINESS MODEL - Maximum security
    pub business_profile: Option<BusinessProfile>,  // None or One, NEVER multiple
    
    // Sponsor fields
    pub sponsor_tier: SponsorTier,
    pub personal_allocation_remaining: u64,
    
    // DAO fields
    pub dao_tier: DaoTier,
    pub dao_contribution: u64,
    pub voting_weight: f64,
    
    // Security
    pub violations: u8,
    pub suspended_until: i64,
    pub permanently_blacklisted: bool,
    pub daily_tx_count: u16,
    pub last_tx_reset: i64,

    // =========================================================================
    // CITIZENSHIP & COUNTRY SANCTIONS
    // =========================================================================
    // ISO 3166-1 alpha-3 country code (e.g., "USA", "GRC", "ISR", "RUS")
    // Used for DAO-imposed sanctions on countries committing atrocities.
    // IMPORTANT: Sanctions punish governments, not individuals permanently.
    // Citizens can still claim reduced UBI - they are victims too.
    
    pub citizenship: [u8; 3],                // ISO 3166-1 alpha-3 code

    // =========================================================================
    // ANNUAL LIVENESS VERIFICATION
    // =========================================================================
    // Every human must prove they are ALIVE once per year.
    // Method: eID biometrics must match LIVE biometric scan.
    // This prevents: dead person fraud, stolen identity abuse.

    pub liveness: LivenessState,

    // =========================================================================
    // PROPOSAL REPUTATION SYSTEM
    // =========================================================================
    // Tracks proposer's history to prevent spam and reward good governance.
    // Reputation calculated automatically from voting results. No manual flagging.
    // Formula: bond_multiplier = 1 + (abs(reputation) / 2)
    
    pub proposal_reputation: i32,             // Reputation score (can be negative)
    pub proposals_created: u32,               // Total proposals submitted
    pub proposals_passed: u32,                // Proposals that passed
    pub proposals_rejected: u32,              // Proposals rejected (but reached quorum)
    pub proposals_expired: u32,               // Proposals that failed to reach quorum

    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BusinessProfile {
    pub name: String,                      // On-chain business name
    pub category: BusinessCategory,
    pub tax_id: Option<String>,
    pub metadata_uri: String,               // IPFS/Arweave pointer to divisions
    pub created_at: i64,
    pub is_verified: bool,                 // DAO verification
    pub is_active: bool,
    pub volume_processed: u128,
    pub employee_count: u32,               // Just count, details off-chain
    pub last_metadata_update: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BusinessInfo {
    pub name: String,
    pub category: BusinessCategory,
    pub tax_id: Option<String>,
    pub metadata_uri: String,               // Required: points to off-chain data
}

// Off-chain metadata structure (stored on IPFS/Arweave)
// {
//     "mainBusiness": "John's Services",
//     "owner": "pubkey...",
//     "divisions": [
//         {"name": "John's Plumbing", "type": "plumbing", "established": "2025-01-01"},
//         {"name": "John's Electric", "type": "electrical", "established": "2025-06-01"},
//         {"name": "John's HVAC", "type": "hvac", "established": "2025-09-01"}
//     ],
//     "employees": [
//         {"wallet": "pubkey...", "name": "Alice", "role": "Plumber", "canTransact": true},
//         {"wallet": "pubkey...", "name": "Bob", "role": "Electrician", "canTransact": true}
//     ],
//     "locations": [
//         {"address": "Main St 1", "type": "headquarters"},
//         {"address": "Second Ave 5", "type": "branch"}
//     ],
//     "verifiedByDAO": false,
//     "lastUpdated": "2025-11-26T12:00:00Z"
// }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum BusinessCategory {
    Retail,
    Food,
    Services,       // Plumber, electrician, etc
    Professional,   // Lawyer, consultant, etc
    Healthcare,
    Education,
    Entertainment,
    Technology,
    Manufacturing,
    Other,
}

// NOTE: MerchantType enum removed - no longer needed with simple OR logic
// Both retail (many customers) and service (high volume) merchants qualify
// via the same simple thresholds

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, PartialOrd)]
pub enum MerchantTier {
    None,
    Emerging,
    Small,
    Medium,
    Large,
    Enterprise,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, PartialOrd)]
pub enum SponsorTier {
    None,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, PartialOrd)]
pub enum DaoTier {
    None,
    Supporter,
    Contributor,
    Builder,
    Sustainer,
    Founder,
}

// ============================================================================
// ANNUAL LIVENESS VERIFICATION STRUCTURES
// ============================================================================
// Philosophy: ALL countries are treated equally. ANY government could be hostile.
// We verify the HUMAN is alive, not that the government says they're alive.
//
// Process:
// 1. User reads biometric data from their eID card (iris, fingerprint, face)
// 2. User performs LIVE biometric scan (cannot be faked by dead person)
// 3. System compares eID biometrics with live scan
// 4. Match = proof of life, account remains active for another year
//
// This is government-agnostic: we use their ID infrastructure but verify independently.

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct LivenessState {
    // Verification status
    pub is_verified: bool,                    // Currently verified as alive
    pub last_verification: i64,               // Timestamp of last successful verification
    pub next_verification_due: i64,           // When next verification is required
    pub verification_count: u32,              // Total successful verifications

    // Biometric identity (privacy-preserving hashes only)
    pub biometric_type: BiometricType,        // What biometric is on user's eID
    pub eid_biometric_hash: [u8; 32],         // SHA-256 of biometric from eID card
    pub eid_issuing_country: [u8; 3],         // ISO 3166-1 alpha-3 country code
    pub eid_expiry: i64,                      // eID card expiry date

    // Anti-fraud
    pub failed_attempts: u8,                  // Failed verification attempts today
    pub last_failed_attempt: i64,             // Timestamp of last failure
    pub locked_until: i64,                    // If locked out, when can retry

    // Challenge-response for live verification
    pub current_challenge: [u8; 32],          // Random challenge for this verification
    pub challenge_issued_at: i64,             // When challenge was issued
    pub challenge_expires_at: i64,            // Challenge expiry (5 minutes)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Default)]
pub enum BiometricType {
    #[default]
    None,           // Not yet registered
    Fingerprint,    // Most common on eID cards
    Iris,           // Higher security, less common
    Face,           // 3D face scan
    Multi,          // Multiple biometrics on eID
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum LivenessStatus {
    Active,         // Verified and within validity period
    Warning,        // Approaching expiry (60 days)
    Expired,        // Past due, in grace period (30 days)
    Suspended,      // Grace period expired, account frozen
    Locked,         // Too many failed attempts, temporarily locked
}

// Input structure for verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BiometricProof {
    pub eid_biometric_hash: [u8; 32],        // Hash of biometric read from eID card
    pub live_biometric_hash: [u8; 32],       // Hash of LIVE biometric scan
    pub biometric_type: BiometricType,       // Type of biometric used
    pub challenge_response: [u8; 32],        // Response to challenge (proves freshness)
    pub eid_country: [u8; 3],                // Country that issued eID
    pub eid_expiry: i64,                     // eID expiry timestamp
    pub timestamp: i64,                       // When this proof was generated
    pub device_attestation: [u8; 64],        // Device signature (prevents replay)
}

// ============================================================================
// VALIDATION CONTEXTS
// ============================================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 512, seeds = [b"protocol"], bump)]
    pub protocol_state: Account<'info, ProtocolState>,
    
    #[account(init, payer = authority, mint::decimals = 9, mint::authority = protocol_state)]
    pub mint: Account<'info, Mint>,
    
    /// CHECK: Treasury for fees
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,
    
    /// CHECK: Gas pool PDA
    #[account(mut)]
    pub gas_pool: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(age_in_days: u64, civic_pass_proof: Vec<u8>, biometric_hash: [u8; 32])]
pub struct RegisterUser<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 1536,  // UserState size
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,

    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,

    // Biometric deduplication - PDA seeded by biometric hash
    // If this hash already exists, init will FAIL Ã¢â€ â€™ duplicate prevented
    // This is the ONLY fraud prevention needed - same fingerprints = same person = reject
    #[account(
        init,
        payer = user,
        space = 8 + 80,  // BiometricEntry size
        seeds = [b"biometric", biometric_hash.as_ref()],
        bump
    )]
    pub biometric_entry: Account<'info, BiometricEntry>,

    #[account(
        mut,
        seeds = [b"biometric_registry"],
        bump = biometric_registry.bump
    )]
    pub biometric_registry: Account<'info, BiometricRegistry>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: Civic Pass verification
    pub civic_pass: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct RegisterBusiness<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateBusiness<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveBusiness<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct BusinessTransaction<'info> {
    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub sender_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"user", recipient.key().as_ref()],
        bump
    )]
    pub recipient_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub treasury_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    pub signer: Signer<'info>,
    
    /// CHECK: Recipient
    pub recipient: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateMerchantStatus<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferWithFee<'info> {
    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump = sender_state.bump
    )]
    pub sender_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"user", recipient.key().as_ref()],
        bump
    )]
    pub recipient_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,
    
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub treasury_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    pub signer: Signer<'info>,
    
    /// CHECK: Recipient wallet
    pub recipient: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimDaily<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,

    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    /// Optional: Country sanction account for user's citizenship
    /// If provided and active, UBI will be reduced according to sanction rate
    pub country_sanction: Option<Account<'info, CountrySanction>>,

    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EmergencyControl<'info> {
    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,

    pub authority: Signer<'info>,
}

// ============================================================================
// COUNTRY SANCTIONS CONTEXTS
// ============================================================================

#[derive(Accounts)]
#[instruction(country_code: [u8; 3])]
pub struct ImposeCountrySanction<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 3 + 1 + 8 + 260 + 32 + 8 + 8 + 8 + 32 + 8 + 8 + 1,  // CountrySanction size
        seeds = [b"country_sanction", country_code.as_ref()],
        bump
    )]
    pub country_sanction: Account<'info, CountrySanction>,

    #[account(
        mut,
        seeds = [b"sanction_registry"],
        bump = sanction_registry.bump
    )]
    pub sanction_registry: Account<'info, SanctionRegistry>,

    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LiftCountrySanction<'info> {
    #[account(
        mut,
        seeds = [b"country_sanction", country_sanction.country_code.as_ref()],
        bump = country_sanction.bump
    )]
    pub country_sanction: Account<'info, CountrySanction>,

    #[account(
        mut,
        seeds = [b"sanction_registry"],
        bump = sanction_registry.bump
    )]
    pub sanction_registry: Account<'info, SanctionRegistry>,

    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeSanctionRegistry<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 4 + 4 + 8 + 1,  // SanctionRegistry size
        seeds = [b"sanction_registry"],
        bump
    )]
    pub sanction_registry: Account<'info, SanctionRegistry>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// ============================================================================
// DAO PROPOSAL CONTEXTS
// ============================================================================

#[derive(Accounts)]
#[instruction(proposal_type: ProposalType)]
pub struct CreateProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = 8 + 8 + 32 + 1 + 1 + 64 + 32 + 8 + 1 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 32 + 3 + 8 + 1 + 1, // Proposal size + spam fields
        seeds = [b"proposal", &proposal_registry.next_proposal_id.to_le_bytes()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        seeds = [b"proposal_registry"],
        bump = proposal_registry.bump
    )]
    pub proposal_registry: Account<'info, ProposalRegistry>,

    #[account(
        init,
        payer = proposer,
        space = 8 + 8,  // Simple escrow
        seeds = [b"bond_escrow", &proposal_registry.next_proposal_id.to_le_bytes()],
        bump
    )]
    /// CHECK: PDA for holding bond
    pub bond_escrow: AccountInfo<'info>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", proposer.key().as_ref()],
        bump
    )]
    pub proposer_state: Account<'info, UserState>,

    #[account(mut)]
    pub proposer_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        init,
        payer = voter,
        space = 8 + 32 + 8 + 1 + 8 + 1,  // VoteRecord size
        seeds = [b"vote", proposal.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,

    #[account(
        seeds = [b"user", voter.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,

    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        seeds = [b"proposal_registry"],
        bump = proposal_registry.bump
    )]
    pub proposal_registry: Account<'info, ProposalRegistry>,

    #[account(
        mut,
        seeds = [b"bond_escrow", &proposal.id.to_le_bytes()],
        bump
    )]
    /// CHECK: PDA for holding bond
    pub bond_escrow: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"user", proposal.proposer.as_ref()],
        bump
    )]
    pub proposer_state: Account<'info, UserState>,

    #[account(mut)]
    pub proposer_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeProposalRegistry<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 8 + 4 + 8 + 8 + 1,  // ProposalRegistry size
        seeds = [b"proposal_registry"],
        bump
    )]
    pub proposal_registry: Account<'info, ProposalRegistry>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// ============================================================================
// LIVENESS VERIFICATION CONTEXTS
// ============================================================================

#[derive(Accounts)]
pub struct RequestLivenessChallenge<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyAnnualLiveness<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReportFailedLiveness<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,

    pub user: Signer<'info>,
}

// ============================================================================
// GAS POOL VALIDATION CONTEXTS
// ============================================================================

#[derive(Accounts)]
pub struct InitializeGasPool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 256,
        seeds = [b"gas_pool_state"],
        bump
    )]
    pub gas_pool_state: Account<'info, GasPoolState>,
    
    #[account(
        seeds = [b"protocol"],
        bump = protocol_state.bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SponsorGasPool<'info> {
    #[account(
        mut,
        seeds = [b"gas_pool_state"],
        bump = gas_pool_state.bump
    )]
    pub gas_pool_state: Account<'info, GasPoolState>,
    
    #[account(
        init_if_needed,
        payer = sponsor_wallet,
        space = 8 + 128,
        seeds = [b"sponsor", sponsor_wallet.key().as_ref()],
        bump
    )]
    pub sponsor_record: Account<'info, SponsorRecord>,
    
    #[account(
        mut,
        seeds = [b"user", sponsor_wallet.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    /// CHECK: Gas pool vault PDA that holds SOL
    #[account(
        mut,
        seeds = [b"gas_pool_vault"],
        bump
    )]
    pub gas_pool_vault: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub sponsor_wallet: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimGasSubsidy<'info> {
    #[account(
        mut,
        seeds = [b"gas_pool_state"],
        bump = gas_pool_state.bump
    )]
    pub gas_pool_state: Account<'info, GasPoolState>,
    
    #[account(
        seeds = [b"user", user_wallet.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"sponsor", user_wallet.key().as_ref()],
        bump = sponsor_record.bump
    )]
    pub sponsor_record: Account<'info, SponsorRecord>,
    
    #[account(
        mut,
        seeds = [b"tx_tracker", user_wallet.key().as_ref()],
        bump = tx_tracker.bump
    )]
    pub tx_tracker: Account<'info, TransactionTracker>,
    
    /// CHECK: Gas pool vault PDA
    #[account(
        mut,
        seeds = [b"gas_pool_vault"],
        bump
    )]
    pub gas_pool_vault: UncheckedAccount<'info>,
    
    /// CHECK: User wallet to receive subsidy
    #[account(mut)]
    pub user_wallet: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RecordTransaction<'info> {
    #[account(
        mut,
        seeds = [b"tx_tracker", user.key().as_ref()],
        bump = tx_tracker.bump
    )]
    pub tx_tracker: Account<'info, TransactionTracker>,
    
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct GasPoolEmergencyControl<'info> {
    #[account(
        mut,
        seeds = [b"gas_pool_state"],
        bump = gas_pool_state.bump
    )]
    pub gas_pool_state: Account<'info, GasPoolState>,
    
    pub authority: Signer<'info>,
}

// ============================================================================
// EVENTS
// ============================================================================

#[event]
pub struct ProtocolInitialized {
    pub mint: Pubkey,
    pub treasury: Pubkey,
    pub gas_pool: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct UserRegistered {
    pub user: Pubkey,
    pub retroactive_claim: u64,
    pub age_in_days: u64,
    pub is_human: bool,
    pub biometric_hash: [u8; 32],
    pub accessible_amount: u64,          // Child protection: immediately accessible
    pub locked_amount: u64,              // Child protection: locked until 18
    pub timestamp: i64,
}

#[event]
pub struct ChildTurned18 {
    pub user: Pubkey,
    pub unlocked_amount: u64,            // Total amount unlocked from locked_balance
    pub timestamp: i64,
}

// ============================================================================
// COUNTRY SANCTIONS EVENTS
// ============================================================================

#[event]
pub struct CountrySanctionImposed {
    pub country_code: [u8; 3],
    pub sanction_rate: u64,
    pub duration_days: u64,
    pub reason: String,
    pub evidence_hash: [u8; 32],
    pub proposal_id: u64,
    pub votes_for: u64,
    pub votes_against: u64,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

#[event]
pub struct CountrySanctionLifted {
    pub country_code: [u8; 3],
    pub lift_reason: String,
    pub original_end_timestamp: i64,
    pub actual_end_timestamp: i64,
}

#[event]
pub struct SanctionedClaimReduced {
    pub user: Pubkey,
    pub country_code: [u8; 3],
    pub original_amount: u64,
    pub reduced_amount: u64,
    pub sanction_rate: u64,
    pub timestamp: i64,
}

#[event]
pub struct BusinessRegistered {
    pub human_owner: Pubkey,
    pub business_name: String,
    pub category: BusinessCategory,
    pub metadata_uri: String,
    pub timestamp: i64,
}

#[event]
pub struct BusinessMetadataUpdated {
    pub business_owner: Pubkey,
    pub business_name: String,
    pub old_metadata_uri: String,
    pub new_metadata_uri: String,
    pub timestamp: i64,
}

#[event]
pub struct BusinessRemoved {
    pub human_owner: Pubkey,
    pub business_name: String,
    pub timestamp: i64,
}

#[event]
pub struct MerchantStatusUpgraded {
    pub merchant: Pubkey,
    pub new_tier: MerchantTier,
    pub unique_customers: u32,
    pub monthly_volume: u128,
    pub timestamp: i64,
}

#[event]
pub struct MerchantStatusDecayed {
    pub merchant: Pubkey,
    pub from_tier: MerchantTier,
    pub to_tier: MerchantTier,
    pub timestamp: i64,
}

#[event]
pub struct TransferCompleted {
    pub sender: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub activity_status: String,  // "active", "normal", "inactive", "dormant"
    pub timestamp: i64,
}

#[event]
pub struct BusinessTransactionCompleted {
    pub human_owner: Pubkey,
    pub business_name: String,
    pub recipient: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub metadata: Option<String>,
    pub timestamp: i64,
}

#[event]
pub struct EmployeeCountUpdated {
    pub business_owner: Pubkey,
    pub business_name: String,
    pub employee_count: u32,
    pub timestamp: i64,
}

#[event]
pub struct DailyClaimed {
    pub user: Pubkey,
    pub amount: u64,
    pub accessible_portion: u64,         // Child protection: goes to accessible_balance
    pub locked_portion: u64,             // Child protection: goes to locked_balance
    pub is_child: bool,
    pub timestamp: i64,
}

#[event]
pub struct BusinessRegistrationFrozen {
    pub frozen: bool,
    pub authority: Pubkey,
    pub timestamp: i64,
}

// ============================================================================
// GAS POOL EVENTS
// ============================================================================

#[event]
pub struct GasPoolInitialized {
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct SponsorshipReceived {
    pub sponsor: Pubkey,
    pub amount: u64,
    pub personal_allocation: u64,
    pub pool_contribution: u64,
    pub new_tier: SponsorTier,
    pub timestamp: i64,
}

#[event]
pub struct SponsorTierUpgraded {
    pub sponsor: Pubkey,
    pub from_tier: SponsorTier,
    pub to_tier: SponsorTier,
    pub total_contributed: u64,
    pub timestamp: i64,
}

#[event]
pub struct GasSubsidyClaimed {
    pub user: Pubkey,
    pub amount: u64,
    pub from_personal: bool,
    pub tx_type: TransactionType,
    pub timestamp: i64,
}

#[event]
pub struct GasPoolEmergency {
    pub reason: EmergencyReason,
    pub daily_distributed: u64,
    pub timestamp: i64,
}

#[event]
pub struct GasPoolResumed {
    pub resumed_by: Pubkey,
    pub timestamp: i64,
}

// ============================================================================
// BIOMETRIC DEDUPLICATION EVENTS
// ============================================================================
// NO country-level events - only individual fraud detection

#[event]
pub struct IndividualFraudDetected {
    pub user: Pubkey,
    pub fraud_type: String,
    pub evidence_hash: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct IndividualBanned {
    pub user: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}

// ============================================================================
// LIVENESS VERIFICATION EVENTS
// ============================================================================

#[event]
pub struct LivenessChallengeIssued {
    pub user: Pubkey,
    pub challenge: [u8; 32],
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct LivenessVerified {
    pub user: Pubkey,
    pub biometric_type: BiometricType,
    pub verification_count: u32,
    pub next_due: i64,
    pub timestamp: i64,
}

#[event]
pub struct LivenessWarning {
    pub user: Pubkey,
    pub days_until_expiry: i64,
    pub timestamp: i64,
}

#[event]
pub struct LivenessExpiredWarning {
    pub user: Pubkey,
    pub grace_days_remaining: i64,
    pub timestamp: i64,
}

#[event]
pub struct LivenessVerificationLockedOut {
    pub user: Pubkey,
    pub failed_attempts: u8,
    pub locked_until: i64,
    pub timestamp: i64,
}

// ============================================================================
// DAO PROPOSAL EVENTS
// ============================================================================

#[event]
pub struct ProposalCreated {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub bond_amount: u64,
    pub voting_ends_at: i64,
}

#[event]
pub struct VoteCast {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub choice: u8,
    pub total_votes: u64,
}

#[event]
pub struct ProposalFinalized {
    pub proposal_id: u64,
    pub status: ProposalStatus,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub votes_abstain: u64,
    pub bond_returned: bool,
}

#[event]
pub struct BondReturned {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct BondForfeited {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub amount: u64,
}

// ============================================================================
// ERROR CODES
// ============================================================================

#[error_code]
pub enum ErrorCode {
    #[msg("Must be verified human via Civic Pass")]
    MustBeVerifiedHuman,
    
    #[msg("Only humans can claim UBI")]
    OnlyHumansCanClaim,
    
    #[msg("Business already exists - only one per human allowed")]
    BusinessAlreadyExists,
    
    #[msg("No business registered")]
    NoBusinessRegistered,
    
    #[msg("Business not active")]
    BusinessNotActive,
    
    #[msg("Business registration is frozen")]
    BusinessRegistrationFrozen,
    
    #[msg("Metadata URI too long (max 128 chars)")]
    MetadataUriTooLong,
    
    #[msg("Metadata update too soon (once per day)")]
    MetadataUpdateTooSoon,
    
    #[msg("Not authorized")]
    Unauthorized,
    
    #[msg("User is suspended")]
    UserSuspended,
    
    #[msg("User is permanently blacklisted")]
    UserBlacklisted,
    
    #[msg("Nothing to claim")]
    NothingToClaim,
    
    #[msg("Claim too soon")]
    ClaimTooSoon,
    
    #[msg("Invalid Civic Pass")]
    InvalidCivicPass,
    
    #[msg("Transaction too soon")]
    TransactionTooSoon,

    // =========================================================================
    // CHILD PROTECTION ERRORS
    // =========================================================================

    #[msg("Transfer exceeds accessible balance - locked funds cannot be spent until age 18")]
    TransferExceedsAccessibleBalance,

    // =========================================================================
    // COUNTRY SANCTIONS ERRORS
    // =========================================================================

    #[msg("Sanction rate too low - minimum 5% to ensure citizens still receive some UBI")]
    SanctionRateTooLow,

    #[msg("Sanction reason too long - maximum 256 characters")]
    SanctionReasonTooLong,

    #[msg("Sanction duration too long - maximum 1 year")]
    SanctionDurationTooLong,

    #[msg("No active sanction for this country")]
    SanctionNotActive,

    #[msg("Daily limit exceeded")]
    DailyLimitExceeded,

    // =========================================================================
    // LIVENESS VERIFICATION ERRORS
    // =========================================================================

    #[msg("Annual liveness verification required - please verify you are alive")]
    LivenessVerificationRequired,

    #[msg("Liveness verification temporarily locked - too many failed attempts")]
    LivenessVerificationLocked,

    #[msg("Only humans can verify liveness")]
    OnlyHumansCanVerify,

    #[msg("No verification challenge issued - request a challenge first")]
    NoChallengeIssued,

    #[msg("Verification challenge has expired - request a new one")]
    ChallengeExpired,

    #[msg("Biometric proof is too old")]
    ProofTooOld,

    #[msg("Biometric proof timestamp is in the future")]
    ProofFromFuture,

    #[msg("eID card has expired - please renew your identity document")]
    EidExpired,

    #[msg("Invalid challenge response - verification failed")]
    InvalidChallengeResponse,

    #[msg("Biometric mismatch - live scan does not match eID biometrics")]
    BiometricMismatch,

    #[msg("Biometric type mismatch - must use same biometric type as registered")]
    BiometricTypeMismatch,

    #[msg("Identity mismatch - this is not the same person who registered")]
    IdentityMismatch,

    #[msg("Invalid device attestation - verification must be from trusted device")]
    InvalidDeviceAttestation,

    // =========================================================================
    // BIOMETRIC DEDUPLICATION ERRORS
    // =========================================================================

    #[msg("Biometric hash already registered - one account per person")]
    BiometricAlreadyRegistered,

    #[msg("Invalid biometric hash format")]
    InvalidBiometricHash,

    // =========================================================================
    // GAS POOL ERRORS
    // =========================================================================

    #[msg("Gas pool is paused - emergency mode active")]
    GasPoolPaused,

    #[msg("Gas pool balance too low")]
    GasPoolLowBalance,

    #[msg("Gas pool emergency pause triggered")]
    GasPoolEmergencyPause,

    #[msg("Daily gas subsidy limit exceeded - try again tomorrow")]
    DailySubsidyLimitExceeded,

    #[msg("Subsidy cooldown active - wait 60 seconds")]
    SubsidyCooldownActive,

    #[msg("Transfer amount too small for gas subsidy (minimum 100 AGORA)")]
    TransferTooSmallForSubsidy,

    // =========================================================================
    // DAO PROPOSAL ERRORS
    // =========================================================================

    #[msg("Proposal is not active")]
    ProposalNotActive,

    #[msg("Voting period has ended")]
    VotingEnded,

    #[msg("Voting period has not ended yet")]
    VotingNotEnded,

    #[msg("Not a verified human - only verified humans can vote")]
    NotVerifiedHuman,

    #[msg("Invalid vote choice - must be 0 (no), 1 (yes), or 2 (abstain)")]
    InvalidVoteChoice,

    #[msg("Proposer is banned - reputation too low to create proposals")]
    ProposerBanned,
}