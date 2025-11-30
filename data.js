/**
 * AGORA Protocol - Centralized Data Source
 * 
 * Currently uses static mock data.
 * TODO: Replace with Solana RPC calls when deployed to devnet/mainnet.
 * 
 * Usage: Include this script in any HTML page, then access AGORA_DATA object.
 */

const AGORA_DATA = {
    // =====================
    // TREASURY - SOL GAS POOL
    // =====================
    gasPool: {
        totalBalance: 7500,             // SOL available
        usedBalance: 2500,              // SOL used
        subsidizedUsers: 75000,         // Users benefiting
        avgMonthlyUsage: 150,           // SOL per month
        activeSponsors: 50,             // Current sponsors
        
        // Sponsor tiers
        tiers: {
            bronze: { amount: 1, monthlyLimit: 800, bonus: 20 },      // 20% = 0.2 SOL
            silver: { amount: 10, monthlyLimit: 6000, bonus: 15 },    // 15% = 1.5 SOL
            gold: { amount: 100, monthlyLimit: 40000, bonus: 10 },    // 10% = 10 SOL
            platinum: { amount: 1000, monthlyLimit: 200000, bonus: 5 }, // 5% = 50 SOL
            diamond: { amount: 10000, monthlyLimit: 1200000, bonus: 3 } // 3% = 300 SOL
        },
        
        // Top sponsors
        sponsors: [
            { name: 'Solana Foundation', tier: 'diamond', amount: 10000 },
            { name: 'Anonymous Whale', tier: 'platinum', amount: 1000 },
            { name: 'CryptoForGood DAO', tier: 'gold', amount: 100 },
            { name: 'DeFi Alliance', tier: 'gold', amount: 100 }
        ]
    },

    // =====================
    // TREASURY - DAO GOVERNANCE
    // =====================
    daoTreasury: {
        solBalance: 120.0,              // SOL in DAO treasury
        totalSpent: 0.0,                // All-time SOL spent
        totalVoters: 60,                // Funding contributors
        
        // Voting thresholds
        votingTiers: {
            tier1: { maxAmount: 1, duration: '24h', quorum: 20 },
            tier2: { maxAmount: 10, duration: '3 days', quorum: 30 },
            tier3: { maxAmount: Infinity, duration: '7 days', quorum: 50 }
        }
    },

    // =====================
    // TREASURY - AGORA BALANCE
    // =====================
    treasury: {
        agoraBalance: 2400000,          // AGORA tokens in treasury
        inflowLast30Days: 124500,       // AGORA received
        outflowLast30Days: 50000,       // AGORA spent
        lastUpdated: new Date().toISOString()
    },

    // =====================
    // PROPOSALS
    // =====================
    proposals: {
        totalCount: 47,
        activeCount: 3,
        
        // Recent/active proposals
        items: [
            {
                id: 'AGP-47',
                title: 'Fund Mobile App Development',
                status: 'voting',
                type: 'treasury',
                proposer: '7xKXtg...2nP9',
                description: 'Allocate 500,000 AGORA for mobile app development to increase accessibility.',
                requestedAmount: 500000,
                votesYes: 12450,
                votesNo: 6230,
                quorum: 15000,
                endTime: new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString(),
                createdAt: new Date(Date.now() - 4 * 24 * 60 * 60 * 1000).toISOString()
            },
            {
                id: 'AGP-46',
                title: 'Increase Daily UBI to 110 AGORA',
                status: 'review',
                type: 'constitutional',
                proposer: '3mNxPq...8kL2',
                description: 'Proposal to increase daily UBI from 100 to 110 AGORA tokens.',
                requestedAmount: 0,
                votesYes: 0,
                votesNo: 0,
                quorum: 25000,
                endTime: null,
                createdAt: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString()
            },
            {
                id: 'AGP-45',
                title: 'Partner with Global NGO Network',
                status: 'voting',
                type: 'standard',
                proposer: '9pQrSt...4mN7',
                description: 'Establish partnership with NGO network for wider UBI distribution.',
                requestedAmount: 0,
                votesYes: 8400,
                votesNo: 1600,
                quorum: 10000,
                endTime: new Date(Date.now() + 5 * 24 * 60 * 60 * 1000).toISOString(),
                createdAt: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString()
            }
        ],
        
        // DAO Treasury proposals (SOL spending)
        daoProposals: [
            {
                id: '001',
                title: 'Fund RPC Infrastructure (Q1 2025)',
                status: 'voting',
                proposer: '@alice',
                description: 'Fund Helius RPC service for 3 months to ensure reliable blockchain access for all users.',
                requestedAmount: 5.0,
                votesYes: 41,
                votesNo: 13,
                votesAbstain: 6,
                quorum: 30,
                endTime: '3 days',
                tier: 2
            },
            {
                id: '002',
                title: 'Security Audit by OtterSec',
                status: 'voting',
                proposer: '@bob',
                description: 'Comprehensive security audit of smart contracts before mainnet launch.',
                requestedAmount: 25.0,
                votesYes: 38,
                votesNo: 8,
                votesAbstain: 4,
                quorum: 50,
                endTime: '5 days',
                tier: 3
            }
        ]
    },

    // =====================
    // VOTING
    // =====================
    voting: {
        userVotingPower: 1,
        pendingVotes: 2,
        totalVotesCast: 23,
        nextDeadline: new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString()
    },

    // =====================
    // SANCTIONS
    // =====================
    sanctions: {
        activeCount: 2,
        historicalCount: 5,
        
        active: [
            {
                countryCode: 'XYZ',
                countryName: 'Example Country',
                reason: 'Human rights violations',
                evidenceHash: 'QmX7b3...ipfs',
                sanctionRate: 1000,
                imposedAt: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
                expiresAt: new Date(Date.now() + 47 * 24 * 60 * 60 * 1000).toISOString(),
                votesFor: 45000,
                votesAgainst: 12000,
                proposalId: 'AGP-38'
            },
            {
                countryCode: 'ABC',
                countryName: 'Another Country',
                reason: 'Genocide',
                evidenceHash: 'QmY8c4...ipfs',
                sanctionRate: 500,
                imposedAt: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000).toISOString(),
                expiresAt: new Date(Date.now() + 120 * 24 * 60 * 60 * 1000).toISOString(),
                votesFor: 52000,
                votesAgainst: 8000,
                proposalId: 'AGP-32'
            }
        ],
        
        historical: [
            {
                countryCode: 'DEF',
                countryName: 'Reformed Country',
                reason: 'Political persecution',
                wasLifted: true,
                liftReason: 'Democratic reforms implemented',
                duration: 90
            }
        ]
    },

    // =====================
    // PROTOCOL STATS
    // =====================
    protocol: {
        totalUsers: 142500,
        dailyActiveUsers: 89000,
        totalUBIClaimed: 1250000000,
        totalSupply: 5200000000,
        circulatingSupply: 4800000000,
        baseTransactionFee: 116,
        treasuryFeeShare: 50,
        burnShare: 50
    },

    // =====================
    // HELPER FUNCTIONS
    // =====================
    
    formatNumber(num) {
        if (num >= 1000000) {
            return (num / 1000000).toFixed(1).replace(/\.0$/, '') + 'M';
        }
        if (num >= 1000) {
            return (num / 1000).toFixed(1).replace(/\.0$/, '') + 'K';
        }
        return num.toLocaleString();
    },

    formatWithCommas(num) {
        return num.toLocaleString();
    },

    formatSOL(num) {
        return num.toLocaleString() + ' SOL';
    },

    getTimeRemaining(endTime) {
        const total = new Date(endTime) - new Date();
        if (total <= 0) return { expired: true, text: 'Ended' };
        
        const days = Math.floor(total / (1000 * 60 * 60 * 24));
        const hours = Math.floor((total % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
        
        if (days > 0) {
            return { expired: false, text: `${days}d ${hours}h` };
        }
        return { expired: false, text: `${hours}h` };
    },

    getVotePercentage(yes, no) {
        const total = yes + no;
        if (total === 0) return 0;
        return Math.round((yes / total) * 100);
    },

    getVotePercentageWithAbstain(yes, no, abstain) {
        const total = yes + no + abstain;
        if (total === 0) return { yes: 0, no: 0, abstain: 0 };
        return {
            yes: Math.round((yes / total) * 100),
            no: Math.round((no / total) * 100),
            abstain: Math.round((abstain / total) * 100)
        };
    },

    sanctionRateToPercent(rate) {
        return rate / 100;
    },
    
    getGasPoolUsagePercent() {
        const total = this.gasPool.totalBalance + this.gasPool.usedBalance;
        return Math.round((this.gasPool.usedBalance / total) * 100);
    }
};

console.log('AGORA Data loaded. Access via AGORA_DATA object.');
