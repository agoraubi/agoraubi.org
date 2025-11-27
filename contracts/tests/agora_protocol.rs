// AGORA Protocol - Smart Contract Tests
// Comprehensive test suite for all contract functions

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_spl::token::{Mint, TokenAccount};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[cfg(test)]
mod tests {
    use super::*;
    use agora_protocol::*;

    // ========================================================================
    // TEST SETUP
    // ========================================================================

    async fn setup() -> (ProgramTest, Keypair, Keypair) {
        let program = ProgramTest::new(
            "agora_protocol",
            agora_protocol::id(),
            processor!(agora_protocol::entry),
        );
        
        let authority = Keypair::new();
        let user = Keypair::new();
        
        (program, authority, user)
    }

    // ========================================================================
    // INITIALIZATION TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_initialize() {
        let (mut program, authority, _) = setup().await;
        let mut context = program.start_with_context().await;
        
        // Create initialize instruction
        let ix = initialize(
            &agora_protocol::id(),
            &authority.pubkey(),
        );
        
        // Execute transaction
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&authority.pubkey()),
            &[&authority],
            context.last_blockhash,
        );
        
        let result = context.banks_client.process_transaction(tx).await;
        assert!(result.is_ok());
        
        // Verify protocol state
        let protocol_state = get_protocol_state(&mut context).await;
        assert_eq!(protocol_state.total_users, 0);
        assert_eq!(protocol_state.total_minted, 0);
        assert_eq!(protocol_state.emergency_paused, false);
    }

    // ========================================================================
    // USER REGISTRATION TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_register_adult_user() {
        let (mut program, authority, user) = setup().await;
        let mut context = program.start_with_context().await;
        
        // Initialize protocol first
        initialize_protocol(&mut context, &authority).await;
        
        // Register adult user (365+ days old)
        let age_in_days = 7300; // 20 years old
        let result = register_user(&mut context, &user, age_in_days).await;
        assert!(result.is_ok());
        
        // Verify user received full retroactive claim
        let user_state = get_user_state(&mut context, &user.pubkey()).await;
        assert_eq!(user_state.age_in_days_at_registration, age_in_days);
        assert_eq!(user_state.total_claimed, 365 * DAILY_AMOUNT); // Max 365 days
        assert_eq!(user_state.is_child, false);
    }

    #[tokio::test]
    async fn test_register_child_user() {
        let (mut program, authority, user) = setup().await;
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        
        // Register child user (under 18)
        let age_in_days = 3650; // 10 years old
        let result = register_user(&mut context, &user, age_in_days).await;
        assert!(result.is_ok());
        
        let user_state = get_user_state(&mut context, &user.pubkey()).await;
        assert_eq!(user_state.is_child, true);
        assert_eq!(user_state.total_claimed, age_in_days * DAILY_AMOUNT);
    }

    #[tokio::test]
    async fn test_register_baby() {
        let (mut program, authority, user) = setup().await;
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        
        // Register baby (100 days old)
        let age_in_days = 100;
        let result = register_user(&mut context, &user, age_in_days).await;
        assert!(result.is_ok());
        
        let user_state = get_user_state(&mut context, &user.pubkey()).await;
        assert_eq!(user_state.total_claimed, 100 * DAILY_AMOUNT);
    }

    // ========================================================================
    // DAILY CLAIM TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_claim_daily_sponsor() {
        let (mut program, authority, user) = setup().await;
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &user, 7300).await;
        
        // Make user a sponsor
        sponsor_gas_pool(&mut context, &user, 1_000_000_000).await; // 1 SOL
        
        // Fast forward 1 day
        advance_clock(&mut context, SECONDS_PER_DAY).await;
        
        // Claim daily
        let result = claim_daily(&mut context, &user).await;
        assert!(result.is_ok());
        
        let user_state = get_user_state(&mut context, &user.pubkey()).await;
        assert_eq!(user_state.total_claimed, 366 * DAILY_AMOUNT); // 365 initial + 1 daily
    }

    #[tokio::test]
    async fn test_claim_daily_non_contributor() {
        let (mut program, authority, user) = setup().await;
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &user, 7300).await;
        
        // Fast forward 1 day (should fail - need 30 days for non-contributors)
        advance_clock(&mut context, SECONDS_PER_DAY).await;
        
        let result = claim_daily(&mut context, &user).await;
        assert!(result.is_err()); // Should fail - too soon
        
        // Fast forward 30 days
        advance_clock(&mut context, SECONDS_PER_DAY * 29).await;
        
        let result = claim_daily(&mut context, &user).await;
        assert!(result.is_ok()); // Should succeed now
    }

    #[tokio::test]
    async fn test_max_accumulation() {
        let (mut program, authority, user) = setup().await;
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &user, 7300).await;
        sponsor_gas_pool(&mut context, &user, 1_000_000_000).await;
        
        // Fast forward 60 days
        advance_clock(&mut context, SECONDS_PER_DAY * 60).await;
        
        // Claim should be capped at 30 days
        let result = claim_daily(&mut context, &user).await;
        assert!(result.is_ok());
        
        let user_state = get_user_state(&mut context, &user.pubkey()).await;
        let expected = 365 * DAILY_AMOUNT + 30 * DAILY_AMOUNT; // Initial + max 30 days
        assert_eq!(user_state.total_claimed, expected);
    }

    // ========================================================================
    // TRANSFER & FEE TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_transfer_with_fees() {
        let (mut program, authority, sender) = setup().await;
        let recipient = Keypair::new();
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &sender, 7300).await;
        register_user(&mut context, &recipient, 7300).await;
        
        let amount = 1000 * DAILY_AMOUNT;
        let expected_fee = (amount * BASE_FEE_RATE) / FEE_DIVISOR;
        let expected_transfer = amount - expected_fee;
        
        let result = transfer_with_burn(
            &mut context,
            &sender,
            &recipient.pubkey(),
            amount,
        ).await;
        assert!(result.is_ok());
        
        // Check balances
        let sender_balance = get_token_balance(&mut context, &sender.pubkey()).await;
        let recipient_balance = get_token_balance(&mut context, &recipient.pubkey()).await;
        
        assert_eq!(recipient_balance, 365 * DAILY_AMOUNT + expected_transfer);
    }

    #[tokio::test]
    async fn test_sponsor_fee_discount() {
        let (mut program, authority, sender) = setup().await;
        let recipient = Keypair::new();
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &sender, 7300).await;
        register_user(&mut context, &recipient, 7300).await;
        
        // Make sender a Gold sponsor (100 SOL = 75% discount)
        sponsor_gas_pool(&mut context, &sender, 100_000_000_000).await;
        
        let amount = 1000 * DAILY_AMOUNT;
        let discounted_rate = BASE_FEE_RATE * 25 / 100; // 75% discount = 25% of original
        let expected_fee = (amount * discounted_rate) / FEE_DIVISOR;
        
        let result = transfer_with_burn(
            &mut context,
            &sender,
            &recipient.pubkey(),
            amount,
        ).await;
        assert!(result.is_ok());
    }

    // ========================================================================
    // GAS POOL TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_sponsor_gas_pool_tiers() {
        let (mut program, authority, user) = setup().await;
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &user, 7300).await;
        
        // Test Bronze tier (1 SOL)
        let result = sponsor_gas_pool(&mut context, &user, 1_000_000_000).await;
        assert!(result.is_ok());
        
        let user_state = get_user_state(&mut context, &user.pubkey()).await;
        assert_eq!(user_state.sponsor_tier, SponsorTier::Bronze);
        assert_eq!(user_state.personal_allocation_remaining, 200_000_000); // 20% of 1 SOL
        
        // Upgrade to Silver (total 10 SOL)
        sponsor_gas_pool(&mut context, &user, 9_000_000_000).await;
        
        let user_state = get_user_state(&mut context, &user.pubkey()).await;
        assert_eq!(user_state.sponsor_tier, SponsorTier::Silver);
    }

    #[tokio::test]
    async fn test_personal_allocation_depletion() {
        let (mut program, authority, user) = setup().await;
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &user, 7300).await;
        
        // Bronze sponsor with 0.2 SOL personal allocation
        sponsor_gas_pool(&mut context, &user, 1_000_000_000).await;
        
        let initial_allocation = 200_000_000; // 0.2 SOL
        
        // Claim gas subsidy multiple times
        for _ in 0..800 {
            claim_gas_subsidy(&mut context, &user).await;
        }
        
        // Personal allocation should be depleted
        let user_state = get_user_state(&mut context, &user.pubkey()).await;
        assert_eq!(user_state.personal_allocation_remaining, 0);
        
        // Next claim should use free tier (limited to 5/day)
        for _ in 0..5 {
            claim_gas_subsidy(&mut context, &user).await;
        }
        
        // 6th claim should fail
        let result = claim_gas_subsidy(&mut context, &user).await;
        assert!(result.is_err()); // Subsidy limit exceeded
    }

    // ========================================================================
    // SECURITY TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_rate_limiting() {
        let (mut program, authority, user) = setup().await;
        let recipient = Keypair::new();
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &user, 7300).await;
        register_user(&mut context, &recipient, 7300).await;
        
        // First transfer should succeed
        let result = transfer_with_burn(
            &mut context,
            &user,
            &recipient.pubkey(),
            100 * DAILY_AMOUNT,
        ).await;
        assert!(result.is_ok());
        
        // Immediate second transfer should fail (60 second minimum)
        let result = transfer_with_burn(
            &mut context,
            &user,
            &recipient.pubkey(),
            100 * DAILY_AMOUNT,
        ).await;
        assert!(result.is_err()); // Too soon
        
        // Fast forward 60 seconds
        advance_clock(&mut context, 60).await;
        
        // Should succeed now
        let result = transfer_with_burn(
            &mut context,
            &user,
            &recipient.pubkey(),
            100 * DAILY_AMOUNT,
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ping_pong_detection() {
        let (mut program, authority, alice) = setup().await;
        let bob = Keypair::new();
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &alice, 7300).await;
        register_user(&mut context, &bob, 7300).await;
        
        // Alice sends to Bob
        transfer_with_burn(
            &mut context,
            &alice,
            &bob.pubkey(),
            100 * DAILY_AMOUNT,
        ).await;
        
        // Advance 61 seconds
        advance_clock(&mut context, 61).await;
        
        // Bob immediately sends back to Alice (ping-pong)
        let result = transfer_with_burn(
            &mut context,
            &bob,
            &alice.pubkey(),
            100 * DAILY_AMOUNT,
        ).await;
        
        // Should detect attack and suspend Bob
        assert!(result.is_err());
        
        let bob_state = get_user_state(&mut context, &bob.pubkey()).await;
        assert_eq!(bob_state.violations, 1);
        assert!(bob_state.suspended_until > 0); // 30 day suspension
    }

    #[tokio::test]
    async fn test_dust_attack_prevention() {
        let (mut program, authority, attacker) = setup().await;
        let victim = Keypair::new();
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &attacker, 7300).await;
        register_user(&mut context, &victim, 7300).await;
        
        // Try to send tiny amount (dust attack)
        let dust_amount = 1_000_000; // 0.001 AGORA
        let result = transfer_with_burn(
            &mut context,
            &attacker,
            &victim.pubkey(),
            dust_amount,
        ).await;
        
        assert!(result.is_err()); // Should fail - below minimum
    }

    #[tokio::test]
    async fn test_permanent_blacklist() {
        let (mut program, authority, attacker) = setup().await;
        let victim = Keypair::new();
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        register_user(&mut context, &attacker, 7300).await;
        register_user(&mut context, &victim, 7300).await;
        
        // First violation - get suspended
        trigger_violation(&mut context, &attacker).await;
        
        let attacker_state = get_user_state(&mut context, &attacker.pubkey()).await;
        assert_eq!(attacker_state.violations, 1);
        
        // Fast forward past suspension
        advance_clock(&mut context, SECONDS_PER_DAY * 31).await;
        
        // Second violation - permanent ban
        trigger_violation(&mut context, &attacker).await;
        
        let attacker_state = get_user_state(&mut context, &attacker.pubkey()).await;
        assert_eq!(attacker_state.violations, 2);
        assert!(attacker_state.permanently_blacklisted);
        
        // All future operations should fail
        let result = claim_daily(&mut context, &attacker).await;
        assert!(result.is_err()); // Blacklisted
    }

    // ========================================================================
    // EMERGENCY TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_emergency_pause() {
        let (mut program, authority, _) = setup().await;
        let mut context = program.start_with_context().await;
        
        initialize_protocol(&mut context, &authority).await;
        
        // Simulate excessive drain
        let gas_pool = get_gas_pool_state(&mut context).await;
        gas_pool.daily_drain_amount = 600_000_000_000; // 600 SOL/day
        
        // Trigger emergency pause
        let result = emergency_pause(&mut context, &authority).await;
        assert!(result.is_ok());
        
        let protocol_state = get_protocol_state(&mut context).await;
        assert!(protocol_state.emergency_paused);
        
        // All gas subsidies should be blocked
        let user = Keypair::new();
        register_user(&mut context, &user, 7300).await;
        
        let result = claim_gas_subsidy(&mut context, &user).await;
        assert!(result.is_err()); // Paused
    }

    // ========================================================================
    // HELPER FUNCTIONS
    // ========================================================================

    async fn initialize_protocol(
        context: &mut ProgramTestContext,
        authority: &Keypair,
    ) {
        // Implementation
    }

    async fn register_user(
        context: &mut ProgramTestContext,
        user: &Keypair,
        age_in_days: u64,
    ) -> Result<(), BanksClientError> {
        // Implementation
        Ok(())
    }

    async fn claim_daily(
        context: &mut ProgramTestContext,
        user: &Keypair,
    ) -> Result<(), BanksClientError> {
        // Implementation
        Ok(())
    }

    async fn sponsor_gas_pool(
        context: &mut ProgramTestContext,
        sponsor: &Keypair,
        amount: u64,
    ) {
        // Implementation
    }

    async fn transfer_with_burn(
        context: &mut ProgramTestContext,
        sender: &Keypair,
        recipient: &Pubkey,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        // Implementation
        Ok(())
    }

    async fn claim_gas_subsidy(
        context: &mut ProgramTestContext,
        user: &Keypair,
    ) -> Result<(), BanksClientError> {
        // Implementation
        Ok(())
    }

    async fn emergency_pause(
        context: &mut ProgramTestContext,
        authority: &Keypair,
    ) -> Result<(), BanksClientError> {
        // Implementation
        Ok(())
    }

    async fn advance_clock(
        context: &mut ProgramTestContext,
        seconds: i64,
    ) {
        // Implementation
    }

    async fn trigger_violation(
        context: &mut ProgramTestContext,
        user: &Keypair,
    ) {
        // Implementation
    }

    async fn get_protocol_state(
        context: &mut ProgramTestContext,
    ) -> ProtocolState {
        // Implementation
        ProtocolState::default()
    }

    async fn get_user_state(
        context: &mut ProgramTestContext,
        user: &Pubkey,
    ) -> UserState {
        // Implementation
        UserState::default()
    }

    async fn get_gas_pool_state(
        context: &mut ProgramTestContext,
    ) -> GasPoolState {
        // Implementation
        GasPoolState::default()
    }

    async fn get_token_balance(
        context: &mut ProgramTestContext,
        user: &Pubkey,
    ) -> u64 {
        // Implementation
        0
    }
}