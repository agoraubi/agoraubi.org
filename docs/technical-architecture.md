# AGORA Protocol - Technical Architecture
## Complete System Specification

**Version 1.3.0**  
**November 2025**  
**Status: Ready for Implementation**

---

## Executive Summary

AGORA Protocol implements Universal Basic Income on Solana blockchain, distributing 100 AGORA tokens daily to every verified human. The system uses Civic Pass for identity verification, implements retroactive fairness for early adopters, and includes a self-balancing economic model with personalized burn rates.

**Key Innovation:** Money that cannot be bought - only earned through humanity or economic activity.

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Token Economics](#2-token-economics)
3. [Smart Contract Architecture](#3-smart-contract-architecture)
4. [Identity Verification](#4-identity-verification)
5. [Distribution Mechanism](#5-distribution-mechanism)
6. [Inflation Model](#6-inflation-model)
7. [Governance Structure](#7-governance-structure)
8. [Gas Pool Economics & Security](#8-gas-pool-economics--security)
9. [Merchant Economy & Auto-Detection](#9-merchant-economy--auto-detection)
10. [Security Considerations](#10-security-considerations)
11. [Implementation Roadmap](#11-implementation-roadmap)
12. [Technical Requirements](#12-technical-requirements)

---

## 1. System Overview

### Core Components

```
┌─────────────────────────────────────────────┐
│              AGORA Protocol                  │
├─────────────────────────────────────────────┤
│                                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │  Token   │  │ Identity │  │   DAO    │  │
│  │  Minting │◄─┤  Verify  │  │ Govern   │  │
│  └──────────┘  └──────────┘  └──────────┘  │
│       ▲              ▲             ▲         │
│       │              │             │         │
│  ┌────┴──────────────┴─────────────┴────┐   │
│  │        Solana Blockchain              │   │
│  │   Program ID: AGORA11...111           │   │
│  └───────────────────────────────────────┘   │
│                                               │
└─────────────────────────────────────────────┘
```

### Technology Stack

- **Blockchain:** Solana
- **Smart Contracts:** Rust with Anchor Framework
- **Token Standard:** SPL Token with Token Extensions
- **Identity:** Civic Pass Integration
- **Frontend:** Pure HTML/CSS/JavaScript
- **Treasury Dashboard:** Real-time on-chain data

---

## 2. Token Economics

### 2.1 Distribution Model

#### Initial Claims (Retroactive Fairness)
```
Age-based distribution:
├── Adults (≥365 days old): 36,500 AGORA (1 year retroactive)
├── Children (<365 days old): age_in_days × 100 AGORA
└── Maximum accumulation: 30 days (3,000 AGORA)
```

#### Daily Distribution
- **Amount:** 100 AGORA per verified human per day
- **Frequency:** Claimable anytime (accumulates up to 30 days)
- **Requirement:** Valid Civic Pass verification

### 2.2 Supply Dynamics

```rust
// No maximum supply cap
total_supply = Σ(daily_mint) + Σ(initial_claims) - Σ(burned_tokens)

where:
  daily_mint = verified_users × 100 × days_elapsed
  initial_claims = Σ(retroactive_distributions)
  burned_tokens = transfer_volume × burn_rate × 0.5
```

### 2.3 Personalized Burn Rate

**Base Fee:** 0.05% (5 basis points)

**Behavioral Multipliers:**
```
Merchant (verified):      0.5x  →  0.025% effective fee
Active Spender:          0.8x  →  0.040% effective fee
Normal User:             1.0x  →  0.050% effective fee
Inactive Hoarder:        1.5x  →  0.075% effective fee
Maximum (multi-factor):  3.0x  →  0.150% effective fee
```

**Fee Distribution:**
- 50% burned (deflationary pressure)
- 50% to protocol treasury (operations)

---

## 3. Smart Contract Architecture

### 3.1 Core Program Structure

```rust
// Program ID (Mainnet)
declare_id!("AGORA111111111111111111111111111111111111111");

// Main Contract Modules
pub mod agora_protocol {
    mod token_mint;      // Token creation and minting
    mod user_registry;   // User registration and verification
    mod distribution;    // Daily and retroactive claims
    mod governance;      // DAO voting mechanisms
    mod treasury;        // Fee collection and management
    mod burn;           // Personalized burn rate calculation
}
```

### 3.2 Account Structure

#### Protocol State (PDA)
```rust
pub struct ProtocolState {
    pub authority: Pubkey,           // DAO authority
    pub mint: Pubkey,               // AGORA token mint
    pub treasury: Pubkey,           // Protocol treasury
    pub total_users: u64,           // Registered user count
    pub total_minted: u128,         // Total tokens minted
    pub total_burned: u128,         // Total tokens burned
    pub launch_timestamp: i64,      // Protocol launch time
    pub civic_gateway: Pubkey,      // Civic Pass gateway
    pub bump: u8,                   // PDA bump seed
}
```

#### User Account (PDA)
```rust
pub struct UserState {
    pub is_initialized: bool,
    pub owner: Pubkey,                      // Wallet address
    pub civic_pass: Pubkey,                 // Civic Pass ID
    pub registration_timestamp: i64,
    pub last_claim_timestamp: i64,
    pub total_claimed: u128,
    pub age_in_days_at_registration: u64,
    pub is_child: bool,                     // Under 18
    pub is_merchant: bool,                  // Verified merchant
    pub transaction_count: u64,             // For behavior scoring
    pub last_transaction_timestamp: i64,
    pub hoarding_score: u8,                 // 0-100 scale
    pub spending_frequency: u8,             // 0-100 scale
    pub bump: u8,
}
```

### 3.3 Core Instructions

#### 1. Initialize Protocol
```rust
pub fn initialize(ctx: Context<Initialize>) -> Result<()>
```

#### 2. Register User
```rust
pub fn register_user(
    ctx: Context<RegisterUser>,
    age_in_days: u64,
    civic_pass_proof: Vec<u8>
) -> Result<()>
```

#### 3. Claim Daily UBI
```rust
pub fn claim_daily(ctx: Context<ClaimDaily>) -> Result<()>
```

#### 4. Transfer with Burn
```rust
pub fn transfer_with_burn(
    ctx: Context<TransferWithBurn>,
    amount: u64
) -> Result<()>
```

---

## 4. Identity Verification

### 4.1 Civic Pass Integration

```
User Registration Flow:
1. User connects Solana wallet
2. Initiates Civic Pass verification
3. Completes KYC with government ID
4. Civic issues pass (on-chain proof)
5. AGORA contract verifies pass
6. User account created with initial claim
```

### 4.2 Sybil Resistance

**Multi-layer Protection:**
1. **Civic Pass:** One person = one pass (government ID verified)
2. **Biometric Hash:** Optional fingerprint/face hash (privacy-preserving)
3. **Social Verification:** Future community vouching system
4. **Behavioral Analysis:** Abnormal claiming patterns flagged

---

## 5. Distribution Mechanism

### 5.1 Claiming Process

```rust
pub fn calculate_claimable(user: &UserState) -> u64 {
    let current_time = Clock::get()?.unix_timestamp;
    let seconds_elapsed = current_time - user.last_claim_timestamp;
    let days_elapsed = (seconds_elapsed / 86400) as u64;
    
    // Cap at 30 days accumulation
    let claimable_days = min(days_elapsed, 30);
    
    claimable_days * DAILY_AMOUNT // 100 AGORA
}
```

### 5.2 Child Protection

```rust
pub fn handle_child_account(user: &UserState, current_time: i64) -> Result<()> {
    if user.is_child {
        let age_in_seconds = current_time - user.birth_timestamp;
        let age_in_years = age_in_seconds / (365 * 86400);
        
        match age_in_years {
            0..=12 => {
                // Fully locked, accumulates but cannot transfer
                user.transfer_locked = true;
            },
            13..=15 => {
                // 10% accessible for education/needs
                user.accessible_percentage = 10;
            },
            16..=17 => {
                // 25% accessible for independence prep
                user.accessible_percentage = 25;
            },
            _ => {
                // Full access at 18
                user.is_child = false;
                user.transfer_locked = false;
                user.accessible_percentage = 100;
            }
        }
    }
    Ok(())
}
```

---

## 6. Inflation Model

### 6.1 Mathematical Model

```
Annual Inflation Rate = (New Tokens Minted / Total Supply) × 100

Where:
- New Tokens = (Active Users × 100 × 365) + One-time Retroactive Claims
- Total Supply = Previous Supply + New Tokens - Burned Tokens
```

### 6.2 Projected Inflation Curve

| Year | Inflation Rate | Total Supply | Users    | Key Factors                |
|------|---------------|--------------|----------|----------------------------|
| 1    | ~90%          | 26.2T        | 366M     | High retroactive claims    |
| 5    | 23.5%         | 91.2T        | 473M     | Growth phase              |
| 10   | 12.6%         | 194.8T       | 608M     | Stabilizing               |
| 15   | 8.6%          | 321.4T       | 728M     | Maturing                  |
| 20   | 6.4%          | 466.8T       | 822M     | Near equilibrium          |
| 30   | ~4%           | 820T         | 950M     | Steady state              |

### 6.3 Equilibrium Mechanics

```
At equilibrium:
Daily Mint Rate ≈ Daily Burn Rate

Example (Year 20):
- Daily mint: 822M users × 100 = 82.2B AGORA
- Daily transfer volume: ~1.6T AGORA
- Daily burn (0.025% avg): ~400M AGORA
- Treasury collection: ~400M AGORA
```

---

## 7. Governance Structure

### 7.1 Pure Direct Democracy

**NO GUARDIANS. NO MULTISIGS. COMMUNITY ONLY.**

```
Voting Power Distribution:
├── Phase 1 (Year 0-1): Initial contributors (1 person = 1 vote)
├── Phase 2 (Year 1+): All verified users (1 person = 1 vote)
└── Not token-weighted (prevents plutocracy)
```

### 7.2 Proposal System

```rust
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub amount_requested: u64,
    pub recipient: Pubkey,
    pub vote_start: i64,
    pub vote_end: i64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub status: ProposalStatus,
    pub execution_timestamp: Option<i64>,
}

pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
}
```

### 7.3 Voting Parameters

| Tier | Treasury Amount | Duration | Quorum | Pass Threshold |
|------|----------------|----------|---------|---------------|
| 1    | <1 SOL         | 24 hours | 20%     | >50%          |
| 2    | 1-10 SOL       | 3 days   | 30%     | >50%          |
| 3    | >10 SOL        | 7 days   | 40%     | >66%          |
| Constitutional | Any    | 14 days  | 50%     | >75%          |

---

## 8. Gas Pool Economics & Security

### 8.1 Problem Statement

Every Solana transaction requires SOL for gas fees (~0.00025 SOL). This creates a barrier for new users, especially in developing nations. Without a solution, AGORA cannot achieve true universal access.

### 8.2 SOL Gas Pool System

#### Pool Structure
```rust
pub struct SOLGasPool {
    pub pool_pda: Pubkey,              // Program-derived account (no private key)
    pub total_deposited: u64,          // Total SOL contributed
    pub total_distributed: u64,        // SOL used for gas subsidies
    pub available_balance: u64,        // Current available SOL
    pub sponsors: Vec<SponsorRecord>,  // List of all sponsors
    pub distribution_rules: Rules,     // Who gets what
}
```

**Key Properties:**
- 100% trustless - PDA controlled
- No human gatekeepers
- Transparent on-chain
- Separate from DAO treasury

### 8.3 Sponsor Tier System

#### Tiered Personal Allocation Model

| Tier | Contribution | Personal Use | To Pool | TX Budget | Fee Discount |
|------|-------------|--------------|---------|-----------|--------------|
| 🥉 Bronze | 1 SOL | 20% (0.2 SOL) | 80% | 800 TX | 25% |
| 🥈 Silver | 10 SOL | 15% (1.5 SOL) | 85% | 6,000 TX | 50% |
| 🥇 Gold | 100 SOL | 10% (10 SOL) | 90% | 40,000 TX | 75% |
| 🏆 Platinum | 1,000 SOL | 5% (50 SOL) | 95% | 200,000 TX | 100% |
| 💎 Diamond | 10,000 SOL | 3% (300 SOL) | 97% | 1.2M TX | 100% + perks |

**Why Tiered Percentages?**
- Small sponsors need higher % for daily use
- Large sponsors are likely institutions/merchants
- Prevents any single actor from draining pool
- Creates sustainable economics

#### Implementation
```rust
pub fn calculate_personal_allocation(amount: u64) -> (u64, u64) {
    let personal_percentage = match amount {
        1..=9 => 20,          // Bronze
        10..=99 => 15,        // Silver
        100..=999 => 10,      // Gold
        1000..=9999 => 5,     // Platinum
        _ => 3,               // Diamond
    };
    
    let personal = amount * personal_percentage / 100;
    let to_pool = amount - personal;
    (personal, to_pool)
}
```

### 8.4 User Access Tiers

```rust
pub enum UserTier {
    NonContributor {
        claim_frequency: Days(30),
        daily_tx_limit: 5,
        gas_coverage: FromPoolOnly,
    },
    Sponsor {
        tier: SponsorTier,
        claim_frequency: Days(1),
        daily_tx_limit: PersonalAllocation,
        gas_coverage: Priority,
    },
}
```

**Access Rules:**
1. **Non-contributors:** Monthly claims, 5 TX/day limit
2. **Sponsors:** Daily claims, use personal allocation
3. **When personal depleted:** Drop to free tier

### 8.5 Attack Vectors & Mitigations

#### 8.5.1 Ping-Pong Attack
**Attack:** Two accounts rapidly exchange tokens to drain gas
```
A → B → A → B (repeat thousands of times)
```

**Mitigation:**
```rust
pub struct AntiPingPong {
    min_time_between_tx: i64,     // 60 seconds minimum
    same_recipient_limit: u8,     // Max 5 TX to same address/hour
    
    fn detect_ping_pong(&self, tx: &Transaction) -> bool {
        // Check if A→B followed by B→A within 5 minutes
        if self.is_return_transaction(tx) {
            return true;  // DETECTED
        }
        false
    }
}
```

#### 8.5.2 Circular Trading Attack
**Attack:** A→B→C→D→A circular flow to drain pool
```
Complex circular patterns harder to detect than simple ping-pong
```

**Mitigation:**
```rust
pub struct NetworkAnalyzer {
    transaction_graph: Graph<Pubkey, Transaction>,
    
    fn detect_cycles(&self, max_depth: u8) -> Vec<Cycle> {
        // Use DFS to find cycles up to depth N
        // If cycle completes in <1 hour = SUSPICIOUS
        let cycles = self.find_strongly_connected_components();
        cycles.filter(|c| c.time_span < 3600)
    }
}
```

#### 8.5.3 Dust Attack
**Attack:** Thousands of micro-transactions (<10 AGORA) to drain gas
```
Send 1 AGORA × 10,000 times = 2.5 SOL drained
```

**Mitigation:**
```rust
pub const MIN_TX_FOR_SUBSIDY: u64 = 100_000_000_000; // 100 AGORA

fn should_subsidize(tx: &Transaction) -> bool {
    if tx.amount < MIN_TX_FOR_SUBSIDY {
        return false;  // Too small, no subsidy
    }
    true
}
```

#### 8.5.4 Sybil Attack
**Attack:** Create 1000 fake accounts to claim gas subsidies

**Mitigation:**
- Civic Pass requirement (1 person = 1 account)
- Behavioral analysis
- Social graph verification
- Progressive verification levels

### 8.6 Rate Limiting System

```rust
pub struct RateLimiter {
    // Per-user limits
    hourly_limit: u8,          // Max 10 TX/hour
    daily_limit: u8,           // Max 50 TX/day
    min_tx_interval: i64,      // 60 seconds between TX
    
    // For subsidized transactions
    subsidized_daily_limit: u8,    // Max 5 subsidized/day
    min_amount_for_subsidy: u64,   // 100 AGORA minimum
}

impl RateLimiter {
    fn check_limits(&self, user: &User, tx: &Transaction) -> Result<()> {
        // Check time since last TX
        if tx.timestamp - user.last_tx < self.min_tx_interval {
            return Err(ErrorCode::TooSoon);
        }
        
        // Check daily limits
        if user.tx_count_today >= self.daily_limit {
            return Err(ErrorCode::DailyLimitExceeded);
        }
        
        // Check subsidy eligibility
        if user.subsidized_today >= self.subsidized_daily_limit {
            tx.subsidy_eligible = false;  // Pay your own gas
        }
        
        Ok(())
    }
}
```

### 8.7 Penalty System

**Zero Tolerance Policy:**
```rust
pub enum Violation {
    FirstViolation => Suspension(Days(30)),
    SecondViolation => PermanentBlacklist,
}
```

**No appeals. No exceptions. No mercy.**

**What triggers penalties:**
- Ping-pong patterns
- Circular trading
- Dust attacks
- Rate limit violations
- Anomalous behavior

### 8.8 Emergency Brake System

```rust
pub struct EmergencyBrake {
    daily_drain_limit: u64,        // Max 100 SOL/day normal
    emergency_threshold: u64,      // 500 SOL/day = emergency
    
    fn check_pool_health(&self) -> PoolStatus {
        let drain_rate = self.calculate_24h_drain_rate();
        
        match drain_rate {
            0..=100 => PoolStatus::Healthy,
            101..=499 => PoolStatus::Warning,
            _ => {
                // AUTO TRIGGER EMERGENCY
                self.pause_all_subsidies();
                self.alert_dao();
                PoolStatus::Emergency
            }
        }
    }
}
```

### 8.9 Economic Sustainability

#### Pool Longevity Calculation
```
Initial pool: 10,000 SOL
Users: 10,000 monthly claimers
Cost per user: 0.003 SOL/month
Monthly burn: 30 SOL
Runway: 333 months (27+ years)
```

#### Growth Projections
```
Year 1: 100 sponsors × avg 100 SOL = 10,000 SOL
Year 2: 1,000 sponsors × avg 50 SOL = 50,000 SOL
Year 3: 10,000 sponsors × avg 20 SOL = 200,000 SOL
Total: 260,000 SOL = Covers 1M users for 20+ years
```

### 8.10 Implementation Priority

**Phase 1 (Launch):** Basic pool + tier system
**Phase 2 (Month 1):** Rate limiting + ping-pong detection
**Phase 3 (Month 2):** Network analysis + circular detection
**Phase 4 (Month 3):** Full security suite + emergency brake

---

## 9. Merchant Economy & Auto-Detection

### 9.1 Merchant Problem Statement

Merchants are critical for AGORA adoption but face challenges:
- Setup complexity and paperwork
- Transaction fees eating into margins  
- Gas fee management
- Integration difficulties
- Cash flow management with crypto volatility

**Solution:** Automatic merchant detection with progressive benefits - no applications, no waiting, just rewards for being a merchant.

### 9.2 Automatic Merchant Detection Algorithm

#### Core Detection Logic
```rust
pub fn calculate_merchant_status(user: &UserState) -> MerchantStatus {
    // Key metrics
    let inflow_outflow_ratio = user.total_received / user.total_sent.max(1);
    let unique_payers = user.unique_senders_30d;
    let tx_frequency = user.daily_tx_average;
    let payment_regularity = calculate_payment_regularity(&user.tx_history);
    
    // Merchant score calculation
    let merchant_score = (
        inflow_outflow_ratio.min(100.0) * 0.4 +        // 40% weight
        (unique_payers as f64 / 10.0).min(100.0) * 0.3 + // 30% weight  
        tx_frequency.min(100.0) * 0.2 +                 // 20% weight
        payment_regularity * 0.1                        // 10% weight
    );
    
    // Auto-tier assignment
    match merchant_score {
        0.0..=10.0 => MerchantStatus::Regular,
        10.1..=25.0 => MerchantStatus::Emerging,
        25.1..=50.0 => MerchantStatus::Small,
        50.1..=75.0 => MerchantStatus::Medium,
        75.1..=90.0 => MerchantStatus::Large,
        _ => MerchantStatus::Enterprise,
    }
}
```

#### Detection Thresholds

| Tier | Min Ratio | Min Unique Customers | Min Daily TX | Monthly Volume |
|------|-----------|---------------------|--------------|----------------|
| Emerging | 5× | 25 | 10 | <1K AGORA |
| Small | 10× | 50 | 20 | <10K AGORA |
| Medium | 25× | 500 | 50 | <100K AGORA |
| Large | 50× | 5,000 | 200 | <1M AGORA |
| Enterprise | 100× | 50,000 | 1,000 | 1M+ AGORA |

### 9.3 Merchant Benefits Package

#### Economic Benefits
```rust
pub struct MerchantEconomicBenefits {
    // Fee structure
    outgoing_fee_discount: u8,      // 50-100% based on tier
    bulk_payment_discount: u8,      // Extra 10% on batch
    
    // Special rates
    employee_payment_rate: u8,      // Always 0% for payroll
    supplier_payment_rate: u8,      // Reduced rate
    
    // Cashback program
    customer_rewards_pool: bool,    // Can offer cashback
    loyalty_points_system: bool,    // Issue merchant tokens
}
```

#### Benefit Tiers

| Merchant Tier | Outgoing Fee | Batch Discount | Employee Payments | API Calls/Month |
|--------------|--------------|----------------|-------------------|-----------------|
| Emerging | 25% off | 5% extra | 50% off | 1,000 |
| Small | 50% off | 10% extra | Free | 10,000 |
| Medium | 75% off | 15% extra | Free | 100,000 |
| Large | 100% off | 20% extra | Free + subsidy | 1,000,000 |
| Enterprise | 100% off | 25% extra | Free + subsidy + priority | Unlimited |

#### Operational Benefits
```rust
pub struct MerchantOperationalBenefits {
    // Payment features
    batch_payments: bool,           // Pay multiple recipients in 1 TX
    recurring_payments: bool,       // Automated subscriptions
    instant_settlement: bool,       // No waiting period
    payment_scheduling: bool,       // Future-dated payments
    
    // Integration tools
    pos_terminal_app: bool,         // Free POS software
    qr_code_generator: bool,        // Dynamic QR codes
    payment_links: bool,            // Shareable payment URLs
    invoice_system: bool,           // Built-in invoicing
    
    // Analytics
    dashboard_access: bool,         // Merchant dashboard
    sales_analytics: bool,         // Revenue tracking
    customer_insights: bool,        // Buyer behavior data
    export_capabilities: bool,      // CSV/JSON/XML export
    
    // Support
    priority_support: bool,         // 24/7 helpline
    dedicated_manager: bool,        // Personal contact
    onboarding_assistance: bool,    // Setup help
    custom_integration: bool,       // API customization
}
```

### 9.4 Employee & Payroll System

#### Automatic Employee Detection
```rust
pub fn detect_employee_relationship(
    merchant: &UserState,
    recipient: &UserState,
) -> bool {
    // Regular payments (weekly/biweekly/monthly)
    let payment_pattern = analyze_payment_frequency(
        &merchant.payments_to[recipient.pubkey]
    );
    
    // Consistent amounts (salary)
    let amount_consistency = calculate_amount_variance(
        &merchant.payments_to[recipient.pubkey]
    );
    
    payment_pattern.is_regular() && amount_consistency < 0.1
}
```

#### Payroll Benefits
- **Zero fees** on all employee payments
- **Batch processing** - pay all employees in one transaction
- **Gas subsidies** for employees from merchant's pool
- **Automated scheduling** - set and forget monthly payments
- **Tax reporting** exports for accounting

### 9.5 DAO Treasury Incentive System

#### Why Contribute to DAO Treasury?

Current problem: Gas pool has clear benefits, DAO treasury has none.

#### DAO Contributor Benefits
```rust
pub struct DaoContributorBenefits {
    // Governance power
    voting_weight_multiplier: f64,    // 1.0× to 2.0× based on contribution
    proposal_rights: ProposalTier,    // Can propose without waiting
    veto_participation: bool,          // Can participate in emergency votes
    
    // Economic benefits
    fee_discount: u8,                 // Up to 50% (not 100% - reserved for gas pool)
    priority_tx_processing: bool,      // Skip mempool queues
    
    // Access benefits  
    early_feature_access: bool,       // Beta features first
    governance_dashboard: bool,        // Advanced analytics
    
    // Recognition
    contributor_badge: DaoTier,        // On-chain recognition
    public_acknowledgment: bool,       // Listed in UI/docs
}
```

#### DAO Contribution Tiers

| Tier | Contribution | Voting Weight | Fee Discount | Special Rights |
|------|-------------|---------------|--------------|----------------|
| Supporter | 0.1+ SOL | 1.0× | 10% | Basic voting |
| Contributor | 1+ SOL | 1.1× | 20% | Proposal comments |
| Builder | 10+ SOL | 1.25× | 30% | Create proposals |
| Sustainer | 100+ SOL | 1.5× | 40% | Priority queue |
| Founder | 1000+ SOL | 2.0× | 50% | Emergency votes |

### 9.6 Merchant Onboarding Flow

#### Automatic Progressive Onboarding
```
Day 1: Regular user
↓ (Receives more than sends)
Day 7: Pattern detected → "Emerging Merchant" status
↓ (Reaches 10× ratio, 50 customers)
Day 30: "Small Merchant" → Benefits activated automatically
↓ (Growth continues)
Day 90: "Medium Merchant" → More benefits unlocked
↓ (Sustained growth)
Year 1: "Large Merchant" → Full benefits package
```

#### Notification System
```rust
pub fn check_merchant_progression(user: &UserState) -> Option<Notification> {
    let current = user.merchant_status;
    let projected = calculate_merchant_status(user);
    
    if projected > current {
        Some(Notification {
            title: "Congratulations! Merchant Status Upgraded",
            message: format!(
                "You're now a {} Merchant! New benefits unlocked:",
                projected
            ),
            benefits: list_new_benefits(current, projected),
            action: "View Dashboard",
        })
    } else if close_to_next_tier(user) {
        Some(Notification {
            title: "You're close to the next merchant tier!",
            message: format!(
                "Only {} more customers to become {} Merchant",
                customers_needed(user),
                next_tier(current)
            ),
        })
    } else {
        None
    }
}
```

### 9.7 Integration Tools

#### REST API for Merchants
```javascript
// Payment processing
POST /api/v1/merchant/payment
{
  "recipient": "wallet_address",
  "amount": 100.50,
  "currency": "AGORA",
  "reference": "INV-2025-001",
  "webhook": "https://merchant.com/callback"
}

// Batch payments
POST /api/v1/merchant/batch
{
  "payments": [
    {"to": "employee1", "amount": 5000},
    {"to": "employee2", "amount": 4500},
    {"to": "supplier1", "amount": 10000}
  ]
}

// Analytics
GET /api/v1/merchant/analytics
{
  "period": "monthly",
  "metrics": ["revenue", "customers", "average_transaction"]
}
```

#### SDK Libraries
- JavaScript/TypeScript
- Python
- PHP (for WooCommerce)
- Java (for enterprise)
- Mobile SDKs (iOS/Android)

### 9.8 Economic Impact Modeling

#### Merchant Adoption Curve
```
Month 1: 10 merchants (early adopters)
Month 3: 100 merchants (word of mouth)
Month 6: 1,000 merchants (network effects)
Year 1: 10,000 merchants (critical mass)
Year 2: 100,000 merchants (mainstream)
```

#### Transaction Volume Projection
```
Regular users: 2-3 TX/day average
Merchants: 50-200 TX/day average
Impact: 1000 merchants = 100,000 regular users in TX volume
```

#### Fee Revenue vs Benefits Cost
```rust
fn calculate_merchant_economics() -> Economics {
    // Merchants pay less fees but generate more volume
    let lost_fee_revenue = merchant_count * average_discount * base_fee;
    let gained_volume_revenue = merchant_tx_volume * regular_fee;
    let network_growth_value = network_effect_multiplier * merchant_count;
    
    Economics {
        direct_loss: lost_fee_revenue,
        direct_gain: gained_volume_revenue,
        network_value: network_growth_value,
        net_benefit: gained_volume_revenue + network_growth_value - lost_fee_revenue,
    }
}
```

### 9.9 Implementation Priority

**Phase 1 (Launch):** Basic detection algorithm
**Phase 2 (Month 1):** Automatic tier assignment
**Phase 3 (Month 2):** Benefit distribution system
**Phase 4 (Month 3):** API & integration tools
**Phase 5 (Month 6):** Full merchant dashboard

---

## 10. Security Considerations

### 10.1 Attack Vectors & Mitigations

| Attack Vector | Description | Mitigation |
|--------------|------------|------------|
| Sybil Attack | Creating multiple fake accounts | Civic Pass verification + one human = one account |
| Business Spam Attack | Creating thousands of fake businesses | Max 1 business per human account |
| Storage Bloat | Overloading chain with business data | Business limit + size constraints |
| Double Claiming | Claiming UBI multiple times | On-chain timestamp tracking |
| Governance Capture | Taking control of voting | One person one vote + quorum requirements |
| Smart Contract Bugs | Exploiting code vulnerabilities | Formal verification + audits |
| Oracle Manipulation | Attacking Civic Pass data | Multiple data sources for Civic |
| Front-running | Transaction order manipulation | Solana's parallel execution model |
| Infinite Mint Bug | Unlimited token creation | Hard-coded daily limits per user |
| Ping-pong Trading | A→B→A rapid trades for gas | 30-day suspension → permanent ban |
| Circular Trading | A→B→C→A wash trading | Network analysis + unique customer requirements |
| Merchant Status Gaming | Faking merchant metrics | Volume-based detection + time requirements |

### 10.2 Business Spam Attack Prevention (Critical)

#### The Attack
Without limits, a malicious actor could:
```rust
// ATTACK SCENARIO: Create unlimited businesses
for i in 0..10000 {
    register_business(format!("FakeBusiness_{}", i));
}

// Consequences:
// - Blockchain storage bloat (each business = ~500 bytes)
// - Performance degradation (iteration over businesses)
// - Confusion in marketplace (finding real businesses)
// - Potential DoS on indexers and explorers
```

#### The Defense
```rust
pub fn register_business() -> Result<()> {
    // CRITICAL: Hard limit of 1 business per human
    require!(
        user.business_profile.is_none(),
        ErrorCode::BusinessAlreadyExists
    );
    
    // Additional protections:
    // - Each business tied to verified human (Civic Pass)
    // - Businesses cannot claim UBI (only humans)
    // - Businesses cannot create other businesses
    // - All transactions traced to human owner
    // - Sub-divisions handled off-chain (IPFS/Arweave)
}
```

#### Why 1 is the Right Limit
- **Real-world accuracy:**
  - Most entities have divisions under ONE legal business
  - Example: "John's Services" with plumbing/electrical divisions
  - Matches actual business structure
  
- **Maximum security:**
  - Cannot create multiple spam businesses per account
  - Storage costs minimal (500 bytes per user)
  - Performance optimal (no iteration)
  
- **Flexibility preserved:**
  - Unlimited divisions via off-chain metadata
  - IPFS/Arweave for branch/location data
  - Employee management still possible

#### Off-Chain Metadata Structure
```json
{
  "main_business": "John's Services",
  "divisions": [
    {"name": "John's Plumbing", "type": "plumbing"},
    {"name": "John's Electric", "type": "electrical"}
  ],
  "locations": [
    {"address": "123 Main St", "type": "headquarters"},
    {"address": "456 Oak Ave", "type": "branch"}
  ]
}
```

#### Future Flexibility
```rust
// DAO can enable multi-business for specific users if needed
pub struct ProtocolState {
    pub single_business_enforced: bool,  // Currently true
}

// Special cases (large franchises) can petition DAO
pub fn dao_grant_multi_business(user: Pubkey) -> Result<()> {
    // Requires governance vote for exception
    // But default remains 1 for security
}
```

### 10.4 Overdraft Impossibility - Why You Can't Spend More Than You Have

#### The Question
"Can you spend more AGORA than you have in your account?"

#### The Answer: ABSOLUTELY IMPOSSIBLE ❌

This is guaranteed by multiple layers of protection, making overdraft mathematically impossible, not just "very difficult."

#### Layer 1: Solana Token Program (Unbreakable)
```rust
// Solana's built-in token program checks BEFORE any transfer
token::transfer(ctx, amount)?;

// What happens internally:
pub fn transfer(from: Account, to: Account, amount: u64) {
    // AUTOMATIC CHECK by Solana (not our code)
    if from.balance < amount {
        return Err(TokenError::InsufficientFunds);  // STOPS HERE
    }
    
    // Only executes if sufficient balance
    from.balance -= amount;
    to.balance += amount;
}
```

#### Layer 2: Type System Protection
```rust
// All amounts are u64 (unsigned 64-bit integers)
pub fn transfer(amount: u64)  // CANNOT be negative

// Attempting negative amount:
transfer(-100)  // COMPILATION ERROR - won't even build
```

#### Layer 3: Atomic Transaction Model
```
Scenario: Alice has 100 AGORA, tries to send 150
Step 1: Check balance (100 < 150) ❌
Step 2: ENTIRE TRANSACTION ROLLED BACK
Result: Alice keeps 100, Bob gets 0, nothing happens
```

#### Attack Scenarios - All Fail

##### Attack 1: Integer Overflow Attempt
```rust
// Attacker tries to overflow the balance
balance: 100 AGORA
send_amount: u64::MAX  // 18,446,744,073,709,551,615

// Result: InsufficientFunds error
// Solana checks ACTUAL balance, not mathematical tricks
```

##### Attack 2: Double Spend (Race Condition)
```rust
// User sends same 100 AGORA in two simultaneous transactions
Balance: 100 AGORA

Transaction 1: Send 100 to Bob    // Submitted at 12:00:00.000
Transaction 2: Send 100 to Carol  // Submitted at 12:00:00.000

// Solana processes sequentially (even if submitted simultaneously):
TX1: 100 - 100 = 0 ✓  // Success, balance now 0
TX2: 0 - 100 = ERROR ❌  // Fails, insufficient funds
```

##### Attack 3: Client Modification
```javascript
// Attacker modifies frontend JavaScript
original: await transfer(100);
modified: await transfer(1000000);

// Result: Transaction fails at blockchain
// Frontend is just UI - blockchain enforces rules
```

##### Attack 4: Smart Contract Bug Exploitation
```rust
// Even if our contract had a bug:
pub fn buggy_function() {
    // Accidentally tries to transfer without checking
    transfer_unchecked(user, 1_000_000);  // BUG!
}

// Still fails because:
// 1. Token Program checks balance
// 2. Account doesn't have 1,000,000
// Result: Transaction reverts
```

##### Attack 5: Negative Balance Trick
```rust
// Mathematical attempt to create negative
balance: 10 AGORA
subtract: 20 AGORA
result: -10  // IMPOSSIBLE with u64

// u64 cannot represent negative numbers
// 10 - 20 in u64 = Integer Underflow Error ❌
```

##### Attack 6: Mint Authority Hijack
```rust
// Attacker tries to mint tokens to themselves
mint_to(attacker_wallet, 1_000_000);

// Fails because:
// - Only protocol_state PDA has mint authority
// - PDA requires specific seeds ["protocol"]
// - Cannot forge PDA signature
// Result: Unauthorized error ❌
```

#### Fee Edge Cases

##### Scenario: Exact Balance with Fees
```rust
Balance: 100.00 AGORA
Send: 100.00 AGORA
Fee: 0.05 AGORA (0.05%)
Total needed: 100.05 AGORA

// Transaction FAILS - need 100.05 but have 100.00
// Maximum sendable: 99.95 AGORA
```

##### Solution in Our Contract
```rust
pub fn calculate_max_transfer(balance: u64, fee_rate: u64) -> u64 {
    // Returns maximum that can be sent INCLUDING fees
    balance * FEE_DIVISOR / (FEE_DIVISOR + fee_rate)
}

// Example: 100 AGORA with 0.05% fee
// Max transfer = 100 * 10000 / 10005 = 99.95 AGORA
```

#### Protection Statistics
| Protection Layer | Bypass Possibility | Security Level |
|-----------------|-------------------|----------------|
| Solana Token Program | 0% | Cryptographic |
| Type System (u64) | 0% | Compile-time |
| Atomic Transactions | 0% | Protocol level |
| Account Model | 0% | Blockchain core |
| Our Validations | 0% | Additional safety |

#### Mathematical Proof
```
Theorem: Balance can never go negative
Proof:
1. Let B = current balance (u64 ≥ 0)
2. Let T = transfer amount (u64 ≥ 0)
3. Transfer succeeds iff B ≥ T
4. New balance B' = B - T
5. Since B ≥ T and both are u64:
   B' ≥ 0 always
   
QED: Balance remains non-negative ∎
```

#### Fun Facts

**Q: What if quantum computers break encryption?**
A: They'd break the signature, not the balance check. You still can't spend what doesn't exist.

**Q: What if someone controls 51% of validators?**
A: Solana uses Proof of Stake, not Proof of Work. Even with consensus control, you can't violate token program rules.

**Q: Has any Solana token ever had an overdraft bug?**
A: No. The token program has been battle-tested since 2020 with trillions in volume.

**Q: What about flash loans?**
A: Flash loans must be repaid in the same transaction. Balance checks still apply at each step.

#### Comparison to Traditional Banking
| System | Overdraft Possible | Protection |
|--------|-------------------|------------|
| Traditional Bank | Yes ✓ | Policy (can be overridden) |
| Credit Card | Yes ✓ | Credit limit (soft) |
| AGORA on Solana | No ❌ | Mathematics (unbreakable) |

#### Conclusion
**Spending more AGORA than you own is not just "very difficult" or "highly unlikely" - it is MATHEMATICALLY IMPOSSIBLE.**

This is the fundamental promise of blockchain: rules enforced by mathematics, not trust.

### 10.5 Upgrade Path

```rust
// All upgrades require DAO approval
pub fn upgrade_program(
    ctx: Context<UpgradeProgram>,
    new_program_id: Pubkey,
    migration_plan: MigrationPlan
) -> Result<()> {
    require!(
        ctx.accounts.proposal.status == ProposalStatus::Passed,
        ErrorCode::ProposalNotPassed
    );
    require!(
        ctx.accounts.proposal.yes_votes > ctx.accounts.proposal.no_votes * 3, 
        ErrorCode::InsufficientMajority  // 75% supermajority
    );
    // Execute upgrade...
}
```

---

## 11. Implementation Roadmap

### Phase 1: Foundation (Months 1-2) ✅
- [x] Technical architecture
- [x] Economic modeling
- [x] Manifesto v1.1
- [x] Landing page
- [x] Treasury dashboard
- [ ] Smart contract development

### Phase 2: Development (Months 3-4)
- [ ] Complete Rust smart contracts
- [ ] Civic Pass integration
- [ ] Frontend application
- [ ] Comprehensive testing suite
- [ ] Devnet deployment

### Phase 3: Audit & Testing (Month 5)
- [ ] Internal code review
- [ ] Community bug bounty
- [ ] Professional audit (OtterSec/Neodyme)
- [ ] Stress testing on devnet
- [ ] Fix discovered issues

### Phase 4: Launch (Month 6)
- [ ] Mainnet deployment
- [ ] Initial contributor funding round
- [ ] User registration opens
- [ ] First retroactive claims
- [ ] Daily claiming begins

### Phase 5: Growth (Months 7-12)
- [ ] 10,000+ users onboarded
- [ ] First DAO proposals
- [ ] Merchant adoption program
- [ ] Mobile app development
- [ ] Cross-chain research

---

## 12. Technical Requirements

### 10.1 Infrastructure

**RPC Nodes:**
- Primary: GenesysGo or Helius
- Backup: Public Solana RPC
- Estimated cost: ~$500/month

**Frontend Hosting:**
- GitHub Pages (free)
- Custom domain via Njalla
- CloudFlare CDN (optional)

**Data Storage:**
- On-chain: User accounts, governance
- Arweave: Historical data archives
- IPFS: Documentation and media

### 10.2 Development Stack

```json
{
  "blockchain": {
    "network": "Solana",
    "language": "Rust",
    "framework": "Anchor 0.29.0",
    "sdk": "@solana/web3.js"
  },
  "frontend": {
    "framework": "Vanilla JS",
    "wallet": "@solana/wallet-adapter",
    "identity": "@civic/solana-gateway",
    "ui": "Custom CSS"
  },
  "testing": {
    "unit": "cargo test",
    "integration": "anchor test",
    "frontend": "jest"
  },
  "monitoring": {
    "explorer": "Solscan API",
    "metrics": "Custom dashboard",
    "alerts": "Discord webhooks"
  }
}
```

### 10.3 Resource Estimation

**Transaction Costs:**
- User registration: ~0.01 SOL
- Daily claim: ~0.001 SOL
- Transfer: ~0.00025 SOL

**Storage Costs:**
- User account: ~0.00203 SOL (rent-exempt)
- Protocol state: ~0.01 SOL (rent-exempt)

**At 1M Users:**
- Total storage: ~2,030 SOL
- Daily transactions: ~1,000 SOL/year
- RPC costs: ~$6,000/year

---

## Appendix A: Error Codes

```rust
#[error_code]
pub enum AgoraError {
    #[msg("User already registered")]
    AlreadyRegistered,
    
    #[msg("Invalid Civic Pass")]
    InvalidCivicPass,
    
    #[msg("Nothing to claim")]
    NothingToClaim,
    
    #[msg("Transfer locked (child account)")]
    TransferLocked,
    
    #[msg("Insufficient balance")]
    InsufficientBalance,
    
    #[msg("Invalid proposal")]
    InvalidProposal,
    
    #[msg("Voting period ended")]
    VotingEnded,
    
    #[msg("Already voted")]
    AlreadyVoted,
    
    #[msg("Unauthorized")]
    Unauthorized,
}
```

---

## Appendix B: Behavioral Scoring Algorithm

```rust
pub fn calculate_behavior_score(user: &UserState) -> BehaviorScore {
    let current_time = Clock::get()?.unix_timestamp;
    let days_since_registration = (current_time - user.registration_timestamp) / 86400;
    
    // Transaction frequency (transactions per day average)
    let tx_frequency = user.transaction_count as f64 / days_since_registration as f64;
    
    // Time since last transaction (staleness)
    let days_since_last_tx = (current_time - user.last_transaction_timestamp) / 86400;
    
    // Calculate scores (0-100 scale)
    let spending_score = min(100, (tx_frequency * 10.0) as u8);
    let hoarding_score = min(100, days_since_last_tx as u8);
    
    BehaviorScore {
        spending_frequency: spending_score,
        hoarding_tendency: hoarding_score,
        fee_multiplier: calculate_fee_multiplier(spending_score, hoarding_score),
    }
}

fn calculate_fee_multiplier(spending: u8, hoarding: u8) -> f64 {
    let base = 1.0;
    
    // Reward active spenders
    let spending_bonus = if spending > 80 { -0.2 } 
                        else if spending > 50 { -0.1 } 
                        else { 0.0 };
    
    // Penalize hoarders
    let hoarding_penalty = if hoarding > 80 { 0.5 }
                          else if hoarding > 50 { 0.25 }
                          else { 0.0 };
    
    (base + spending_bonus + hoarding_penalty).max(0.5).min(3.0)
}
```

---

## Appendix C: Retroactive Distribution Formula

```rust
pub fn calculate_initial_claim(age_in_days: u64) -> u64 {
    const DAILY_AMOUNT: u64 = 100_000_000_000; // 100 AGORA (9 decimals)
    const MAX_RETROACTIVE_DAYS: u64 = 365;
    
    if age_in_days >= MAX_RETROACTIVE_DAYS {
        // Adults: Full year retroactive
        MAX_RETROACTIVE_DAYS * DAILY_AMOUNT
    } else {
        // Children: Proportional to age
        age_in_days * DAILY_AMOUNT
    }
}
```

---

## Version History

- **v1.0.0** (Nov 20, 2025): Initial architecture with retroactive model
- **v1.1.0** (Nov 25, 2025): Added inflation model, behavioral scoring, updated governance  
- **v1.2.0** (Nov 26, 2025): Added Gas Pool Economics, sponsor tiers, attack mitigations
- **v1.3.0** (Nov 26, 2025): Added Merchant Economy, auto-detection, DAO incentives

---

**For technical questions:** GitHub Issues  
**For general questions:** Discord  
**For security concerns:** security@agora-protocol.org (PGP available)

---

*"Code is law, but community is justice."*

**END OF DOCUMENT**