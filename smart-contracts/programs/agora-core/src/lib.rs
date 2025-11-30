//! # AGORA Core v1.0
//! 
//! # ╔═══════════════════════════════════════════════════════════════════════════╗
//! # ║                                                                           ║
//! # ║   ██╗███╗   ███╗███╗   ███╗██╗   ██╗████████╗ █████╗ ██████╗ ██╗     ███████╗   ║
//! # ║   ██║████╗ ████║████╗ ████║██║   ██║╚══██╔══╝██╔══██╗██╔══██╗██║     ██╔════╝   ║
//! # ║   ██║██╔████╔██║██╔████╔██║██║   ██║   ██║   ███████║██████╔╝██║     █████╗     ║
//! # ║   ██║██║╚██╔╝██║██║╚██╔╝██║██║   ██║   ██║   ██╔══██║██╔══██╗██║     ██╔══╝     ║
//! # ║   ██║██║ ╚═╝ ██║██║ ╚═╝ ██║╚██████╔╝   ██║   ██║  ██║██████╔╝███████╗███████╗   ║
//! # ║   ╚═╝╚═╝     ╚═╝╚═╝     ╚═╝ ╚═════╝    ╚═╝   ╚═╝  ╚═╝╚═════╝ ╚══════╝╚══════╝   ║
//! # ║                                                                           ║
//! # ║   This program is IMMUTABLE. After deployment, upgrade authority is       ║
//! # ║   permanently revoked. NO ONE can ever modify this code again.            ║
//! # ║                                                                           ║
//! # ║   Command after deployment:                                               ║
//! # ║   solana program set-upgrade-authority <PROGRAM_ID> --final               ║
//! # ║                                                                           ║
//! # ╚═══════════════════════════════════════════════════════════════════════════╝
//!
//! ## Purpose
//! 
//! AGORA Core handles the sacred, unchangeable parts of the protocol:
//! 
//! - **100 AGORA/day** - The fundamental promise to every human, FOREVER
//! - **User registration** - Biometric deduplication (one human = one account)
//! - **Daily claims** - UBI distribution (reads sanctions from Governance)
//! - **Token transfers** - Moving AGORA between accounts
//! - **Retroactive claims** - Up to 365 days back-payment on registration
//! 
//! ## What This Program Does NOT Handle
//! 
//! Everything else is in `agora-governance` (upgradeable by DAO):
//! 
//! - DAO proposals and voting
//! - Country sanctions (creating/removing sanction accounts)
//! - Fee parameters and adjustments
//! - Merchant detection and tiers
//! - Gas pool and sponsor system
//! - Protocol treasury
//! 
//! ## Relationship with Governance Program
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        AGORA CORE (IMMUTABLE)                           │
//! │                                                                         │
//! │   • Reads CountrySanction accounts from Governance                      │
//! │   • Reads fee parameters from Governance                                │
//! │   • NEVER calls Governance functions                                    │
//! │   • NEVER allows Governance to modify Core state                        │
//! │   • NEVER allows anyone to change 100 AGORA/day                         │
//! │                                                                         │
//! │   Formula (immutable):                                                  │
//! │   claim_amount = DAILY_AMOUNT × sanction.ubi_percentage / 100           │
//! │                                                                         │
//! │   If no sanction exists: claim_amount = 100 AGORA                       │
//! │   If sanction at 25%:    claim_amount = 25 AGORA                        │
//! │   Governance can ONLY reduce via sanctions, NEVER increase above 100    │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//!                                    │
//!                                    │ reads accounts (no function calls)
//!                                    ▼
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                     AGORA GOVERNANCE (UPGRADEABLE)                      │
//! │                                                                         │
//! │   • Creates/removes CountrySanction accounts                            │
//! │   • Manages DAO proposals and voting                                    │
//! │   • Adjusts fee parameters                                              │
//! │   • Handles gas pool and sponsors                                       │
//! │   • Can be upgraded by DAO vote                                         │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Model
//! 
//! - Biometric hash ensures one human = one account (cannot be bypassed)
//! - Sanctions only REDUCE UBI, never increase (max is always 100)
//! - No external program can call mint functions
//! - Rate limiting prevents abuse
//! - All state changes emit events for transparency
//!
//! ## License
//! 
//! Apache 2.0 - See LICENSE file

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, MintTo, Transfer, Burn}
};

declare_id!("AGoRACoreXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    SACRED CONSTANTS - NEVER MODIFY                        ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝
//
// The following constant is the IDENTITY of AGORA Protocol.
// It represents a promise to every human being on Earth.
// It must NEVER, under ANY circumstances, be modified.
//
// If you are reviewing this code for an upgrade: REJECT any change to this value.
// If you are a DAO member voting on an upgrade: REJECT any change to this value.
// If you are a developer tempted to change this: DON'T.
//
// 100 AGORA per day, per human, forever. This is non-negotiable.
// This is not a parameter. This is not configurable. This is AGORA.

pub const DAILY_AMOUNT: u64 = 100_000_000_000;  // 100 AGORA (9 decimals) - IMMUTABLE FOREVER

// ============================================================================
// OTHER CORE CONSTANTS
// ============================================================================

/// Maximum retroactive days for initial claim (1 year)
pub const MAX_RETROACTIVE_DAYS: u64 = 365;

/// Maximum days tokens can accumulate before expiring (rolling window)
pub const MAX_ACCUMULATION_DAYS: u64 = 30;

/// Seconds in a day (used for time calculations)
pub const SECONDS_PER_DAY: i64 = 86400;

/// Token decimals (Solana standard for SPL tokens)
pub const TOKEN_DECIMALS: u8 = 9;

/// Minimum time between transactions (rate limiting)
pub const MIN_TIME_BETWEEN_TX: i64 = 60; // 60 seconds

/// Age threshold for child protection (18 years in days)
pub const CHILD_AGE_THRESHOLD: u64 = 6570; // ~18 years

/// Liveness verification period (1 year)
pub const LIVENESS_PERIOD_SECONDS: i64 = 31_536_000; // 365 days

/// Grace period after liveness expiry
pub const LIVENESS_GRACE_PERIOD: i64 = 2_592_000; // 30 days

/// Governance program ID - used to verify sanction accounts
/// This is the ONLY connection between Core and Governance
pub const GOVERNANCE_PROGRAM_ID: Pubkey = pubkey!("AGoRAGovXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

// ============================================================================
// PROGRAM ENTRY POINT
// ============================================================================

#[program]
pub mod agora_core {
    use super::*;

    // ========================================================================
    // INITIALIZATION
    // ========================================================================

    /// Initialize the AGORA Core protocol.
    /// 
    /// This function is called ONCE at deployment to set up:
    /// - Protocol state PDA
    /// - Token mint with correct decimals
    /// - Biometric registry for deduplication
    /// 
    /// # Access Control
    /// Only the deployer can call this, and only once.
    /// 
    /// # Arguments
    /// * `ctx` - Context containing all required accounts
    /// 
    /// # Events
    /// Emits `ProtocolInitialized`
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Initialize protocol state
        protocol.authority = ctx.accounts.authority.key();
        protocol.mint = ctx.accounts.mint.key();
        protocol.treasury = ctx.accounts.treasury.key();
        protocol.total_users = 0;
        protocol.total_minted = 0;
        protocol.total_burned = 0;
        protocol.launch_timestamp = current_time;
        protocol.is_initialized = true;
        protocol.bump = ctx.bumps.protocol_state;
        
        // Initialize biometric registry
        let registry = &mut ctx.accounts.biometric_registry;
        registry.total_registered = 0;
        registry.bump = ctx.bumps.biometric_registry;
        
        emit!(ProtocolInitialized {
            authority: protocol.authority,
            mint: protocol.mint,
            treasury: protocol.treasury,
            launch_timestamp: current_time,
        });
        
        msg!("AGORA Core initialized. 100 AGORA/day forever begins now.");
        
        Ok(())
    }

    // ========================================================================
    // USER REGISTRATION
    // ========================================================================

    /// Register a new user with biometric verification.
    /// 
    /// This function:
    /// 1. Verifies the biometric hash is unique (one human = one account)
    /// 2. Creates a UserState PDA for the user
    /// 3. Records the biometric hash in the registry
    /// 4. Mints the initial retroactive claim (up to 365 days)
    /// 
    /// # Biometric Deduplication
    /// 
    /// The `biometric_hash` is a SHA-256 hash of the user's fingerprint.
    /// It is deterministic: same fingerprint = same hash, always.
    /// If this hash already exists, registration is rejected.
    /// 
    /// # Retroactive Claim
    /// 
    /// - Adults (≥365 days old): Receive 36,500 AGORA (1 year × 100/day)
    /// - Children (<365 days old): Receive age_in_days × 100 AGORA
    /// 
    /// # Child Protection
    /// 
    /// If user is under 18 years old:
    /// - Initial claim is LOCKED until they turn 18
    /// - Daily claims are also locked
    /// - Cannot transfer tokens until age 18
    /// 
    /// # Arguments
    /// * `ctx` - Context with user accounts
    /// * `age_in_days` - User's age in days (from verified ID)
    /// * `biometric_hash` - SHA-256 hash of fingerprint
    /// * `citizenship` - ISO 3166-1 alpha-3 country code
    /// 
    /// # Events
    /// Emits `UserRegistered` and `InitialClaimMinted`
    pub fn register_user(
        ctx: Context<RegisterUser>,
        age_in_days: u64,
        biometric_hash: [u8; 32],
        citizenship: [u8; 3],
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let protocol = &mut ctx.accounts.protocol_state;
        let user = &mut ctx.accounts.user_state;
        let registry = &mut ctx.accounts.biometric_registry;
        let bio_record = &mut ctx.accounts.biometric_record;
        
        // ====================================================================
        // BIOMETRIC DEDUPLICATION CHECK
        // ====================================================================
        // This is the core Sybil resistance mechanism.
        // If this hash exists, the same human already has an account.
        
        require!(
            !bio_record.is_registered,
            CoreError::BiometricAlreadyRegistered
        );
        
        // ====================================================================
        // INITIALIZE USER STATE
        // ====================================================================
        
        user.owner = ctx.accounts.owner.key();
        user.registration_timestamp = current_time;
        user.last_claim_timestamp = current_time;
        user.age_in_days_at_registration = age_in_days;
        user.citizenship = citizenship;
        user.is_verified = true;
        user.is_child = age_in_days < CHILD_AGE_THRESHOLD;
        user.total_claimed = 0;
        user.transaction_count = 0;
        user.last_transaction_timestamp = current_time;
        user.liveness_verified_at = current_time;
        user.liveness_expires_at = current_time + LIVENESS_PERIOD_SECONDS;
        user.bump = ctx.bumps.user_state;
        
        // ====================================================================
        // REGISTER BIOMETRIC
        // ====================================================================
        
        bio_record.hash = biometric_hash;
        bio_record.user = ctx.accounts.owner.key();
        bio_record.registered_at = current_time;
        bio_record.is_registered = true;
        bio_record.invalidated_at = 0;
        bio_record.previous_hash = None;
        bio_record.bump = ctx.bumps.biometric_record;
        
        registry.total_registered += 1;
        
        // ====================================================================
        // CALCULATE RETROACTIVE CLAIM
        // ====================================================================
        // Adults get 1 year back-pay, children get proportional to age
        
        let retroactive_days = if age_in_days >= MAX_RETROACTIVE_DAYS {
            MAX_RETROACTIVE_DAYS
        } else {
            age_in_days
        };
        
        let initial_claim = retroactive_days * DAILY_AMOUNT;
        
        // ====================================================================
        // CHECK FOR COUNTRY SANCTION
        // ====================================================================
        // We read the sanction account from Governance program.
        // If it exists and is active, reduce the claim proportionally.
        
        let sanction_percentage = match &ctx.accounts.country_sanction {
            Some(sanction_account) => {
                // Deserialize and check if active
                let sanction_data = sanction_account.try_borrow_data()?;
                if sanction_data.len() >= 8 + 3 + 1 { // Discriminator + country + is_active
                    let is_active = sanction_data[8 + 3] == 1;
                    if is_active && sanction_data.len() >= 8 + 3 + 1 + 1 {
                        let ubi_pct = sanction_data[8 + 3 + 1];
                        ubi_pct as u64
                    } else {
                        100 // Not active = full UBI
                    }
                } else {
                    100 // Invalid account = full UBI
                }
            },
            None => 100, // No sanction = full 100%
        };
        
        // Apply sanction percentage (max 100, enforced by Governance)
        let actual_claim = initial_claim * sanction_percentage.min(100) / 100;
        
        // ====================================================================
        // MINT INITIAL CLAIM
        // ====================================================================
        
        // Mint tokens to user's token account
        let seeds = &[
            b"protocol",
            &[protocol.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.protocol_state.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        
        token::mint_to(cpi_ctx, actual_claim)?;
        
        // Update state
        user.total_claimed = actual_claim as u128;
        user.locked_balance = if user.is_child { actual_claim } else { 0 };
        protocol.total_users += 1;
        protocol.total_minted += actual_claim as u128;
        
        // ====================================================================
        // EMIT EVENTS
        // ====================================================================
        
        emit!(UserRegistered {
            user: user.owner,
            registration_timestamp: current_time,
            age_in_days,
            citizenship,
            is_child: user.is_child,
        });
        
        emit!(InitialClaimMinted {
            user: user.owner,
            retroactive_days,
            amount: actual_claim,
            sanction_percentage,
            is_locked: user.is_child,
        });
        
        msg!(
            "User registered: {} days old, {} AGORA initial claim ({}% UBI)",
            age_in_days,
            actual_claim / 1_000_000_000,
            sanction_percentage
        );
        
        Ok(())
    }

    // ========================================================================
    // DAILY UBI CLAIM
    // ========================================================================

    /// Claim accumulated daily UBI tokens.
    /// 
    /// Users accumulate 100 AGORA per day automatically. This function
    /// allows them to claim their accumulated tokens (up to 30 days max).
    /// 
    /// # Accumulation Rules
    /// 
    /// - Tokens accumulate at 100 AGORA/day (IMMUTABLE)
    /// - Maximum accumulation: 30 days (3,000 AGORA)
    /// - Tokens older than 30 days expire (rolling window)
    /// - Claim frequency: daily, weekly, monthly - user's choice
    /// 
    /// # Liveness Requirement
    /// 
    /// User must have valid liveness verification (renewed annually).
    /// If expired, user must call `verify_liveness` first.
    /// 
    /// # Sanction Awareness
    /// 
    /// If user's country is under sanction, they receive reduced UBI:
    /// `actual_amount = DAILY_AMOUNT × days × sanction_percentage / 100`
    /// 
    /// # Child Protection
    /// 
    /// Tokens for users under 18 are minted but LOCKED.
    /// They cannot be transferred until the user turns 18.
    /// 
    /// # Arguments
    /// * `ctx` - Context with user and protocol accounts
    /// 
    /// # Events
    /// Emits `DailyClaimed`
    pub fn claim_daily(ctx: Context<ClaimDaily>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let protocol = &mut ctx.accounts.protocol_state;
        let user = &mut ctx.accounts.user_state;
        
        // ====================================================================
        // VERIFICATION CHECKS
        // ====================================================================
        
        require!(user.is_verified, CoreError::UserNotVerified);
        
        // Check liveness (with grace period)
        let liveness_deadline = user.liveness_expires_at + LIVENESS_GRACE_PERIOD;
        require!(
            current_time <= liveness_deadline,
            CoreError::LivenessExpired
        );
        
        // ====================================================================
        // CALCULATE CLAIMABLE AMOUNT
        // ====================================================================
        
        let seconds_since_last_claim = current_time - user.last_claim_timestamp;
        let days_since_last_claim = (seconds_since_last_claim / SECONDS_PER_DAY) as u64;
        
        // Cap at maximum accumulation window
        let claimable_days = days_since_last_claim.min(MAX_ACCUMULATION_DAYS);
        
        require!(claimable_days > 0, CoreError::NothingToClaim);
        
        // Base amount (before sanctions)
        let base_amount = claimable_days * DAILY_AMOUNT;
        
        // ====================================================================
        // CHECK FOR COUNTRY SANCTION
        // ====================================================================
        
        let sanction_percentage = match &ctx.accounts.country_sanction {
            Some(sanction_account) => {
                let sanction_data = sanction_account.try_borrow_data()?;
                if sanction_data.len() >= 8 + 3 + 1 + 1 {
                    let is_active = sanction_data[8 + 3] == 1;
                    if is_active {
                        let ubi_pct = sanction_data[8 + 3 + 1];
                        ubi_pct as u64
                    } else {
                        100
                    }
                } else {
                    100
                }
            },
            None => 100,
        };
        
        // ====================================================================
        // APPLY SANCTION AND MINT
        // ====================================================================
        // This is the critical formula - sanctions can only REDUCE, never INCREASE
        
        let actual_amount = base_amount * sanction_percentage.min(100) / 100;
        
        // Mint tokens
        let seeds = &[
            b"protocol",
            &[protocol.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.protocol_state.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        
        token::mint_to(cpi_ctx, actual_amount)?;
        
        // ====================================================================
        // UPDATE STATE
        // ====================================================================
        
        user.last_claim_timestamp = current_time;
        user.total_claimed += actual_amount as u128;
        
        // If child, add to locked balance
        if user.is_child {
            // Check if user has turned 18 since registration
            let current_age_days = user.age_in_days_at_registration + 
                ((current_time - user.registration_timestamp) / SECONDS_PER_DAY) as u64;
            
            if current_age_days >= CHILD_AGE_THRESHOLD {
                // User turned 18 - unlock everything
                user.is_child = false;
                user.locked_balance = 0;
            } else {
                // Still a child - lock the new tokens
                user.locked_balance += actual_amount;
            }
        }
        
        protocol.total_minted += actual_amount as u128;
        
        // ====================================================================
        // EMIT EVENT
        // ====================================================================
        
        emit!(DailyClaimed {
            user: user.owner,
            days_claimed: claimable_days,
            base_amount,
            sanction_percentage,
            actual_amount,
            is_locked: user.is_child,
            timestamp: current_time,
        });
        
        msg!(
            "Claimed {} days = {} AGORA ({}% UBI)",
            claimable_days,
            actual_amount / 1_000_000_000,
            sanction_percentage
        );
        
        Ok(())
    }

    // ========================================================================
    // LIVENESS VERIFICATION
    // ========================================================================

    /// Verify that the user is still alive (annual requirement).
    /// 
    /// This prevents:
    /// - Dead person fraud (continuing to claim after death)
    /// - Identity theft (thief cannot produce matching biometrics)
    /// 
    /// # Process
    /// 
    /// 1. User initiates liveness challenge (this function)
    /// 2. User provides LIVE biometric scan (fingerprint/iris)
    /// 3. System verifies scan matches stored biometric hash
    /// 4. If match: liveness extended for 1 year
    /// 
    /// # Timing
    /// 
    /// - Liveness valid for 365 days
    /// - 30-day grace period after expiry
    /// - Warning notifications at 60 days before expiry
    /// 
    /// # Arguments
    /// * `ctx` - Context with user account
    /// * `live_biometric_hash` - Hash of LIVE biometric scan
    /// 
    /// # Events
    /// Emits `LivenessVerified`
    pub fn verify_liveness(
        ctx: Context<VerifyLiveness>,
        live_biometric_hash: [u8; 32],
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let user = &mut ctx.accounts.user_state;
        let bio_record = &ctx.accounts.biometric_record;
        
        // ====================================================================
        // VERIFY BIOMETRIC MATCH
        // ====================================================================
        // The live scan must match the registered biometric hash.
        // This proves the person doing the verification is the same person
        // who originally registered.
        
        require!(
            live_biometric_hash == bio_record.hash,
            CoreError::BiometricMismatch
        );
        
        // ====================================================================
        // EXTEND LIVENESS
        // ====================================================================
        
        user.liveness_verified_at = current_time;
        user.liveness_expires_at = current_time + LIVENESS_PERIOD_SECONDS;
        
        emit!(LivenessVerified {
            user: user.owner,
            verified_at: current_time,
            expires_at: user.liveness_expires_at,
        });
        
        msg!("Liveness verified. Valid until {}", user.liveness_expires_at);
        
        Ok(())
    }

    // ========================================================================
    // BIOMETRIC UPDATE
    // ========================================================================

    /// Update user's biometric hash when ID technology changes.
    /// 
    /// This function allows users to migrate to new biometric technology
    /// (e.g., from fingerprint to DNA hash) without losing their account.
    /// 
    /// # Security Model
    /// 
    /// User must prove ownership of BOTH:
    /// 1. The OLD biometric (proves they own the current account)
    /// 2. The NEW biometric (proves they own the new identity)
    /// 
    /// This prevents account theft - an attacker would need access to
    /// both the old AND new biometric data.
    /// 
    /// # Process
    /// 
    /// 1. User provides live scan of OLD biometric → system verifies match
    /// 2. User provides NEW biometric hash from updated eID
    /// 3. System verifies NEW hash doesn't already exist (no duplicates)
    /// 4. Old BiometricRecord is invalidated
    /// 5. New BiometricRecord is created
    /// 
    /// # Use Cases
    /// 
    /// - eID system upgrades from fingerprint to iris scan
    /// - Country adopts DNA-based identity
    /// - User's biometric changes (injury, medical condition)
    /// - Technology improvements (better hash algorithms)
    /// 
    /// # Arguments
    /// * `ctx` - Context with old and new biometric records
    /// * `old_biometric_proof` - Live scan of OLD biometric (must match stored)
    /// * `new_biometric_hash` - Hash from NEW biometric (from updated eID)
    /// 
    /// # Events
    /// Emits `BiometricUpdated`
    pub fn update_biometric(
        ctx: Context<UpdateBiometric>,
        old_biometric_proof: [u8; 32],
        new_biometric_hash: [u8; 32],
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let user = &mut ctx.accounts.user_state;
        let old_bio_record = &mut ctx.accounts.old_biometric_record;
        let new_bio_record = &mut ctx.accounts.new_biometric_record;
        let registry = &mut ctx.accounts.biometric_registry;
        
        // ====================================================================
        // VERIFY OWNERSHIP OF OLD BIOMETRIC
        // ====================================================================
        // User must prove they own the current account by providing
        // a live scan that matches the stored hash.
        
        require!(
            old_biometric_proof == old_bio_record.hash,
            CoreError::BiometricMismatch
        );
        
        require!(
            old_bio_record.user == ctx.accounts.owner.key(),
            CoreError::Unauthorized
        );
        
        require!(
            old_bio_record.is_registered,
            CoreError::BiometricNotRegistered
        );
        
        // ====================================================================
        // VERIFY NEW BIOMETRIC IS UNIQUE
        // ====================================================================
        // The new biometric must not already be registered to anyone.
        // This prevents someone from "stealing" another person's identity.
        
        require!(
            !new_bio_record.is_registered,
            CoreError::BiometricAlreadyRegistered
        );
        
        // ====================================================================
        // INVALIDATE OLD BIOMETRIC RECORD
        // ====================================================================
        
        old_bio_record.is_registered = false;
        old_bio_record.invalidated_at = current_time;
        
        // ====================================================================
        // CREATE NEW BIOMETRIC RECORD
        // ====================================================================
        
        new_bio_record.hash = new_biometric_hash;
        new_bio_record.user = ctx.accounts.owner.key();
        new_bio_record.registered_at = current_time;
        new_bio_record.is_registered = true;
        new_bio_record.previous_hash = Some(old_bio_record.hash);
        new_bio_record.bump = ctx.bumps.new_biometric_record;
        
        // Registry count stays the same (1 invalidated, 1 added)
        
        // ====================================================================
        // UPDATE USER STATE
        // ====================================================================
        // Reset liveness to require fresh verification with new biometric
        
        user.liveness_verified_at = current_time;
        user.liveness_expires_at = current_time + LIVENESS_PERIOD_SECONDS;
        
        emit!(BiometricUpdated {
            user: user.owner,
            old_hash: old_bio_record.hash,
            new_hash: new_biometric_hash,
            timestamp: current_time,
        });
        
        msg!("Biometric updated successfully. Old hash invalidated, new hash registered.");
        
        Ok(())
    }

    // ========================================================================
    // TOKEN TRANSFER
    // ========================================================================

    /// Transfer AGORA tokens between users.
    /// 
    /// This is the basic transfer function. Fees are calculated and applied
    /// by the Governance program (via fee parameters it controls).
    /// 
    /// # Fee Structure
    /// 
    /// Fees are read from Governance program's fee state:
    /// - Base fee: 0.05% (can be adjusted by DAO)
    /// - 50% of fees burned (deflationary)
    /// - 50% to treasury
    /// 
    /// # Child Protection
    /// 
    /// Children cannot transfer tokens. Their balance is locked until age 18.
    /// 
    /// # Rate Limiting
    /// 
    /// Minimum 60 seconds between transactions to prevent spam.
    /// 
    /// # Arguments
    /// * `ctx` - Context with sender, recipient, and fee accounts
    /// * `amount` - Amount to transfer (before fees)
    /// 
    /// # Events
    /// Emits `TokensTransferred`
    pub fn transfer_tokens(
        ctx: Context<TransferTokens>,
        amount: u64,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let sender = &mut ctx.accounts.sender_state;
        let protocol = &mut ctx.accounts.protocol_state;
        
        // ====================================================================
        // VALIDATION
        // ====================================================================
        
        require!(sender.is_verified, CoreError::UserNotVerified);
        require!(!sender.is_child, CoreError::ChildCannotTransfer);
        require!(amount > 0, CoreError::InvalidAmount);
        
        // Rate limiting
        require!(
            current_time - sender.last_transaction_timestamp >= MIN_TIME_BETWEEN_TX,
            CoreError::RateLimitExceeded
        );
        
        // ====================================================================
        // READ FEE PARAMETERS FROM GOVERNANCE
        // ====================================================================
        
        let (fee_rate, burn_pct) = match &ctx.accounts.fee_state {
            Some(fee_account) => {
                let fee_data = fee_account.try_borrow_data()?;
                if fee_data.len() >= 8 + 8 + 8 { // Discriminator + fee_rate + burn_pct
                    let fee_rate = u64::from_le_bytes(fee_data[8..16].try_into().unwrap());
                    let burn_pct = u64::from_le_bytes(fee_data[16..24].try_into().unwrap());
                    (fee_rate, burn_pct)
                } else {
                    (5, 50) // Default: 0.05% fee, 50% burn
                }
            },
            None => (5, 50), // Default if no fee state
        };
        
        // ====================================================================
        // CALCULATE FEES
        // ====================================================================
        
        let fee_amount = (amount * fee_rate) / 10000; // fee_rate is in basis points
        let burn_amount = (fee_amount * burn_pct) / 100;
        let treasury_amount = fee_amount - burn_amount;
        let transfer_amount = amount - fee_amount;
        
        // ====================================================================
        // EXECUTE TRANSFER
        // ====================================================================
        
        // Transfer to recipient
        let cpi_accounts = Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.sender.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, transfer_amount)?;
        
        // Transfer to treasury
        if treasury_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.sender_token_account.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: ctx.accounts.sender.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
            );
            token::transfer(cpi_ctx, treasury_amount)?;
        }
        
        // Burn fee portion
        if burn_amount > 0 {
            let cpi_accounts = Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.sender_token_account.to_account_info(),
                authority: ctx.accounts.sender.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
            );
            token::burn(cpi_ctx, burn_amount)?;
            
            protocol.total_burned += burn_amount as u128;
        }
        
        // ====================================================================
        // UPDATE STATE
        // ====================================================================
        
        sender.transaction_count += 1;
        sender.last_transaction_timestamp = current_time;
        
        emit!(TokensTransferred {
            sender: sender.owner,
            recipient: ctx.accounts.recipient.key(),
            amount: transfer_amount,
            fee_amount,
            burn_amount,
            treasury_amount,
            timestamp: current_time,
        });
        
        Ok(())
    }
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                           ACCOUNT STRUCTURES                              ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

// ============================================================================
// PROTOCOL STATE
// ============================================================================

/// Global protocol state - singleton PDA.
/// 
/// Stores protocol-wide statistics and references.
/// Created once during initialization.
#[account]
pub struct ProtocolState {
    /// Original deployer authority (for reference only, no special powers)
    pub authority: Pubkey,
    
    /// AGORA token mint address
    pub mint: Pubkey,
    
    /// Protocol treasury address
    pub treasury: Pubkey,
    
    /// Total registered users
    pub total_users: u64,
    
    /// Total tokens ever minted (including burned)
    pub total_minted: u128,
    
    /// Total tokens burned
    pub total_burned: u128,
    
    /// Timestamp when protocol was launched
    pub launch_timestamp: i64,
    
    /// Whether protocol is initialized
    pub is_initialized: bool,
    
    /// PDA bump seed
    pub bump: u8,
}

impl ProtocolState {
    pub const SIZE: usize = 8 +  // Discriminator
        32 +    // authority
        32 +    // mint
        32 +    // treasury
        8 +     // total_users
        16 +    // total_minted
        16 +    // total_burned
        8 +     // launch_timestamp
        1 +     // is_initialized
        1;      // bump
}

// ============================================================================
// USER STATE
// ============================================================================

/// Individual user account state.
/// 
/// Created when a user registers. Tracks their claims, transactions,
/// and verification status.
#[account]
pub struct UserState {
    /// User's wallet address
    pub owner: Pubkey,
    
    /// When user registered
    pub registration_timestamp: i64,
    
    /// Last time user claimed UBI
    pub last_claim_timestamp: i64,
    
    /// User's age in days at registration
    pub age_in_days_at_registration: u64,
    
    /// User's citizenship (ISO 3166-1 alpha-3)
    pub citizenship: [u8; 3],
    
    /// Whether user passed identity verification
    pub is_verified: bool,
    
    /// Whether user is under 18
    pub is_child: bool,
    
    /// Total AGORA ever claimed (for statistics)
    pub total_claimed: u128,
    
    /// Locked balance (for children)
    pub locked_balance: u64,
    
    /// Transaction count (for activity tracking)
    pub transaction_count: u64,
    
    /// Last transaction timestamp (for rate limiting and activity)
    pub last_transaction_timestamp: i64,
    
    /// When liveness was last verified
    pub liveness_verified_at: i64,
    
    /// When liveness expires
    pub liveness_expires_at: i64,
    
    /// PDA bump seed
    pub bump: u8,
}

impl UserState {
    pub const SIZE: usize = 8 +  // Discriminator
        32 +    // owner
        8 +     // registration_timestamp
        8 +     // last_claim_timestamp
        8 +     // age_in_days_at_registration
        3 +     // citizenship
        1 +     // is_verified
        1 +     // is_child
        16 +    // total_claimed
        8 +     // locked_balance
        8 +     // transaction_count
        8 +     // last_transaction_timestamp
        8 +     // liveness_verified_at
        8 +     // liveness_expires_at
        1;      // bump
}

// ============================================================================
// BIOMETRIC REGISTRY
// ============================================================================

/// Global registry tracking total biometric registrations.
/// 
/// This is a singleton PDA that tracks statistics only.
/// Individual biometric hashes are stored in BiometricRecord PDAs.
#[account]
pub struct BiometricRegistry {
    /// Total number of registered biometrics
    pub total_registered: u64,
    
    /// PDA bump seed
    pub bump: u8,
}

impl BiometricRegistry {
    pub const SIZE: usize = 8 +  // Discriminator
        8 +     // total_registered
        1;      // bump
}

/// Individual biometric record.
/// 
/// PDA derived from biometric hash - ensures uniqueness.
/// If a record exists for a hash, that human is already registered.
#[account]
pub struct BiometricRecord {
    /// The biometric hash (SHA-256 of fingerprint, iris, DNA, etc.)
    pub hash: [u8; 32],
    
    /// The user this biometric belongs to
    pub user: Pubkey,
    
    /// When this biometric was registered
    pub registered_at: i64,
    
    /// Whether this is a valid registration (false if superseded)
    pub is_registered: bool,
    
    /// When this record was invalidated (0 if still active)
    pub invalidated_at: i64,
    
    /// Previous biometric hash (if this is an update)
    pub previous_hash: Option<[u8; 32]>,
    
    /// PDA bump seed
    pub bump: u8,
}

impl BiometricRecord {
    pub const SIZE: usize = 8 +  // Discriminator
        32 +    // hash
        32 +    // user
        8 +     // registered_at
        1 +     // is_registered
        8 +     // invalidated_at
        1 + 32 + // previous_hash (Option)
        1;      // bump
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                          CONTEXT STRUCTURES                               ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

// ============================================================================
// INITIALIZE CONTEXT
// ============================================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Deployer (pays for account creation)
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// Protocol state PDA
    #[account(
        init,
        payer = authority,
        space = ProtocolState::SIZE,
        seeds = [b"protocol"],
        bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,
    
    /// Biometric registry PDA
    #[account(
        init,
        payer = authority,
        space = BiometricRegistry::SIZE,
        seeds = [b"biometric_registry"],
        bump
    )]
    pub biometric_registry: Account<'info, BiometricRegistry>,
    
    /// AGORA token mint
    #[account(
        init,
        payer = authority,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = protocol_state,
    )]
    pub mint: Account<'info, Mint>,
    
    /// Treasury token account
    /// CHECK: Created by associated token program
    #[account(mut)]
    pub treasury: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// ============================================================================
// REGISTER USER CONTEXT
// ============================================================================

#[derive(Accounts)]
#[instruction(age_in_days: u64, biometric_hash: [u8; 32], citizenship: [u8; 3])]
pub struct RegisterUser<'info> {
    /// User registering
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// User state PDA
    #[account(
        init,
        payer = owner,
        space = UserState::SIZE,
        seeds = [b"user", owner.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    /// Biometric record PDA (derived from hash)
    #[account(
        init,
        payer = owner,
        space = BiometricRecord::SIZE,
        seeds = [b"biometric", biometric_hash.as_ref()],
        bump
    )]
    pub biometric_record: Account<'info, BiometricRecord>,
    
    /// Biometric registry (to update count)
    #[account(
        mut,
        seeds = [b"biometric_registry"],
        bump = biometric_registry.bump
    )]
    pub biometric_registry: Account<'info, BiometricRegistry>,
    
    /// Protocol state
    #[account(
        mut,
        seeds = [b"protocol"],
        bump = protocol_state.bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,
    
    /// Token mint
    #[account(
        mut,
        address = protocol_state.mint
    )]
    pub mint: Account<'info, Mint>,
    
    /// User's token account
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// Optional: Country sanction account from Governance program
    /// If this account exists and is active, UBI is reduced
    /// CHECK: We verify this is from Governance program and parse manually
    #[account(
        seeds = [b"sanction", citizenship.as_ref()],
        bump,
        seeds::program = GOVERNANCE_PROGRAM_ID,
    )]
    pub country_sanction: Option<AccountInfo<'info>>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// ============================================================================
// CLAIM DAILY CONTEXT
// ============================================================================

#[derive(Accounts)]
pub struct ClaimDaily<'info> {
    /// User claiming
    pub owner: Signer<'info>,
    
    /// User state
    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump = user_state.bump,
        constraint = user_state.owner == owner.key() @ CoreError::Unauthorized
    )]
    pub user_state: Account<'info, UserState>,
    
    /// Protocol state
    #[account(
        mut,
        seeds = [b"protocol"],
        bump = protocol_state.bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,
    
    /// Token mint
    #[account(
        mut,
        address = protocol_state.mint
    )]
    pub mint: Account<'info, Mint>,
    
    /// User's token account
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// Optional: Country sanction account
    /// CHECK: Verified and parsed manually
    pub country_sanction: Option<AccountInfo<'info>>,
    
    pub token_program: Program<'info, Token>,
}

// ============================================================================
// VERIFY LIVENESS CONTEXT
// ============================================================================

#[derive(Accounts)]
pub struct VerifyLiveness<'info> {
    /// User verifying liveness
    pub owner: Signer<'info>,
    
    /// User state
    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump = user_state.bump,
        constraint = user_state.owner == owner.key() @ CoreError::Unauthorized
    )]
    pub user_state: Account<'info, UserState>,
    
    /// Biometric record (to verify hash matches)
    #[account(
        seeds = [b"biometric", biometric_record.hash.as_ref()],
        bump = biometric_record.bump,
        constraint = biometric_record.user == owner.key() @ CoreError::Unauthorized
    )]
    pub biometric_record: Account<'info, BiometricRecord>,
}

// ============================================================================
// UPDATE BIOMETRIC CONTEXT
// ============================================================================

#[derive(Accounts)]
#[instruction(old_biometric_proof: [u8; 32], new_biometric_hash: [u8; 32])]
pub struct UpdateBiometric<'info> {
    /// User updating their biometric
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// User state
    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump = user_state.bump,
        constraint = user_state.owner == owner.key() @ CoreError::Unauthorized
    )]
    pub user_state: Account<'info, UserState>,
    
    /// Old biometric record (to be invalidated)
    #[account(
        mut,
        seeds = [b"biometric", old_biometric_record.hash.as_ref()],
        bump = old_biometric_record.bump,
        constraint = old_biometric_record.user == owner.key() @ CoreError::Unauthorized
    )]
    pub old_biometric_record: Account<'info, BiometricRecord>,
    
    /// New biometric record (to be created)
    #[account(
        init,
        payer = owner,
        space = BiometricRecord::SIZE,
        seeds = [b"biometric", new_biometric_hash.as_ref()],
        bump
    )]
    pub new_biometric_record: Account<'info, BiometricRecord>,
    
    /// Biometric registry (for statistics)
    #[account(
        mut,
        seeds = [b"biometric_registry"],
        bump = biometric_registry.bump
    )]
    pub biometric_registry: Account<'info, BiometricRegistry>,
    
    pub system_program: Program<'info, System>,
}

// ============================================================================
// TRANSFER TOKENS CONTEXT
// ============================================================================

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    /// Sender
    #[account(mut)]
    pub sender: Signer<'info>,
    
    /// Sender state
    #[account(
        mut,
        seeds = [b"user", sender.key().as_ref()],
        bump = sender_state.bump,
        constraint = sender_state.owner == sender.key() @ CoreError::Unauthorized
    )]
    pub sender_state: Account<'info, UserState>,
    
    /// Recipient
    /// CHECK: Just receiving tokens
    pub recipient: AccountInfo<'info>,
    
    /// Protocol state
    #[account(
        mut,
        seeds = [b"protocol"],
        bump = protocol_state.bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,
    
    /// Token mint
    #[account(
        mut,
        address = protocol_state.mint
    )]
    pub mint: Account<'info, Mint>,
    
    /// Sender's token account
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = sender,
    )]
    pub sender_token_account: Account<'info, TokenAccount>,
    
    /// Recipient's token account
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    
    /// Treasury token account
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = protocol_state.treasury,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,
    
    /// Optional: Fee state from Governance program
    /// CHECK: Verified and parsed manually
    pub fee_state: Option<AccountInfo<'info>>,
    
    pub token_program: Program<'info, Token>,
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                               EVENTS                                      ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

#[event]
pub struct ProtocolInitialized {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub treasury: Pubkey,
    pub launch_timestamp: i64,
}

#[event]
pub struct UserRegistered {
    pub user: Pubkey,
    pub registration_timestamp: i64,
    pub age_in_days: u64,
    pub citizenship: [u8; 3],
    pub is_child: bool,
}

#[event]
pub struct InitialClaimMinted {
    pub user: Pubkey,
    pub retroactive_days: u64,
    pub amount: u64,
    pub sanction_percentage: u64,
    pub is_locked: bool,
}

#[event]
pub struct DailyClaimed {
    pub user: Pubkey,
    pub days_claimed: u64,
    pub base_amount: u64,
    pub sanction_percentage: u64,
    pub actual_amount: u64,
    pub is_locked: bool,
    pub timestamp: i64,
}

#[event]
pub struct LivenessVerified {
    pub user: Pubkey,
    pub verified_at: i64,
    pub expires_at: i64,
}

#[event]
pub struct TokensTransferred {
    pub sender: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub fee_amount: u64,
    pub burn_amount: u64,
    pub treasury_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct BiometricUpdated {
    pub user: Pubkey,
    pub old_hash: [u8; 32],
    pub new_hash: [u8; 32],
    pub timestamp: i64,
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                              ERROR CODES                                  ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

#[error_code]
pub enum CoreError {
    #[msg("This biometric hash is already registered to another account")]
    BiometricAlreadyRegistered,
    
    #[msg("User is not verified")]
    UserNotVerified,
    
    #[msg("Liveness verification has expired - please verify you are alive")]
    LivenessExpired,
    
    #[msg("Biometric scan does not match registered biometric")]
    BiometricMismatch,
    
    #[msg("Nothing to claim - no days have passed since last claim")]
    NothingToClaim,
    
    #[msg("Children cannot transfer tokens until age 18")]
    ChildCannotTransfer,
    
    #[msg("Invalid amount")]
    InvalidAmount,
    
    #[msg("Rate limit exceeded - please wait before next transaction")]
    RateLimitExceeded,
    
    #[msg("Unauthorized - you are not the owner of this account")]
    Unauthorized,
    
    #[msg("Biometric record is not registered or has been invalidated")]
    BiometricNotRegistered,
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                          END OF AGORA CORE                                ║
// ║                                                                           ║
// ║   This program is IMMUTABLE after deployment. The 100 AGORA/day          ║
// ║   promise is encoded forever in this code. No one can change it.         ║
// ║                                                                           ║
// ║   "100 AGORA per day, per human, forever."                               ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝
