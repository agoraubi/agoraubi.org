# AGORA Protocol

**Universal Basic Income for humanity. Mathematical governance. No central control.**

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Built on Solana](https://img.shields.io/badge/Built%20on-Solana-green)](https://solana.com)
[![UBI: 100 AGORA](https://img.shields.io/badge/UBI-100%20AGORA%2Fday-purple)](https://agoraubi.org)

---

## What is AGORA?

AGORA is an open-source protocol that provides Universal Basic Income (UBI) through blockchain technology. Every verified human receives **100 AGORA tokens daily**, distributed by mathematics, not politics.

> *"The Digital Agora - Where Economics Meets Democracy"*

### Core Principles

- 🌍 **Universal**: Every human has the right to economic participation
- 🔐 **Secure**: Government-verified identity with blockchain distribution
- 🗳️ **Democratic**: One person, one vote (not one dollar, one vote)
- 🔄 **Sustainable**: Mathematical monetary policy, not central bank control
- 📖 **Open Source**: Transparent, auditable, forkable

---

## Key Features

| Feature | Description |
|---------|-------------|
| ✅ **Daily UBI** | 100 AGORA tokens for every verified human |
| ✅ **30-Day Accumulation** | Tokens accumulate for up to 30 days if unclaimed |
| ✅ **Child Protection** | Tokens locked until age 18 to prevent parental abuse |
| ✅ **Liveness Verification** | Annual check prevents zombie accounts |
| ✅ **Ultra-Low Fees** | 0.0001 SOL claim fee, 0.1% transfer fee |
| ✅ **Country Trust Scoring** | Prevents government identity fraud |
| ✅ **Mathematical Governance** | Algorithmic fairness - same rules for everyone |

---

## Why AGORA?

Current economic systems concentrate wealth and power. AGORA creates a **mathematical floor for human dignity** while preserving free markets above that floor.

**This is not charity. This is not socialism. This is mathematical fairness.**

## Philosophy: 1 AGORA = 1 AGORA

We don't measure AGORA in dollars or euros. Like Bitcoin's "1 BTC = 1 BTC", we believe:

- **Value comes from utility**, not speculation
- **Circular economies** are the goal, not fiat conversion
- **Real adoption** means paying for goods and services in AGORA

---

## Technical Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    AGORA Protocol                       │
├─────────────────────────────────────────────────────────┤
│  Layer 1: Identity Verification                         │
│  └── Government APIs (Civic)                            | 
├─────────────────────────────────────────────────────────┤
│  Layer 2: Token Distribution                            │
│  └── 100 AGORA/day per verified human                   │
├─────────────────────────────────────────────────────────┤
│  Layer 3: Economic Policy                               │
│  └── Personalized PI Controller, Burn Mechanism         │
├─────────────────────────────────────────────────────────┤
│  Layer 4: Governance                                    │
│  └── DAO with 1 person = 1 vote                         │
├─────────────────────────────────────────────────────────┤
│  Layer 5: Solana Blockchain                             │
│  └── Smart contracts (Rust/Anchor)                      │
└─────────────────────────────────────────────────────────┘
```

---

## Repository Structure

```
agora-protocol/
├── README.md
├── LICENSE                    # Apache 2.0
├── CONTRIBUTING.md
├── SECURITY.md
├── .gitignore
│
├── contracts/                 # Smart contracts
│   ├── programs/
│   │   ├── agora-token/
│   │   ├── agora-distribution/
│   │   └── agora-governance/
│   ├── tests/
│   └── Anchor.toml
│
├── app/                       # Web frontend
│   ├── src/
│   ├── public/
│   └── package.json
│
├── docs/                      # Documentation
│   ├── whitepaper.md
│   ├── technical-architecture.md
│   ├── manifesto.md
│   └── developer-guide.md
│
└── scripts/
    ├── deploy.sh
    └── test.sh
```

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.17+)
- [Anchor](https://www.anchor-lang.com/) (v0.29+)
- [Node.js](https://nodejs.org/) (v18+)

### Installation

```bash
# Clone the repository
git clone https://github.com/agora-protocol/agora-protocol.git
cd agora-protocol

# Install dependencies
npm install

# Build smart contracts
cd contracts
anchor build

# Run tests
anchor test
```

### Deploy to Devnet

```bash
# Configure for devnet
solana config set --url devnet

# Deploy
anchor deploy --provider.cluster devnet
```

---

## Protocol Fees

AGORA uses ultra-low fees to ensure sustainability:

| Fee Type | Amount | Distribution |
|----------|--------|--------------|
| Claim Fee | 0.0001 SOL (~$0.014) | 50% burn, 50% treasury |
| Transfer Fee | 0.1% | 50% burn, 50% treasury |

All fee parameters are controlled by DAO governance.

---

## Roadmap

### Phase 1: Foundation (Q4 2025)
- [x] Technical architecture
- [x] Manifesto & documentation
- [ ] Core smart contracts
- [ ] Security audit

### Phase 2: Testnet (Q1 2026)
- [ ] Deploy to Solana Devnet
- [ ] Community beta testing
- [ ] Bug bounty program
- [ ] Identity provider integration

### Phase 3: Mainnet (Q2 2026)
- [ ] Mainnet deployment
- [ ] Initial user onboarding
- [ ] Merchant adoption program
- [ ] Governance activation

### Phase 4: Growth (2026+)
- [ ] Multi-country expansion
- [ ] Mobile app
- [ ] Additional identity providers
- [ ] Full decentralization

---

## Documentation

- 📄 [Technical Architecture](docs/technical-architecture.md)
- 📜 [Manifesto](docs/manifesto.md)
- 👨‍💻 [Developer Guide](docs/developer-guide.md)
- 🔐 [Security Policy](SECURITY.md)
- 🤝 [Contributing Guide](CONTRIBUTING.md)

---

## Community

- 🌐 **Website**: [agoraubi.org](https://agoraubi.org)
- 📧 **Email**: agora.ubi@tuta.io

---

## Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) before submitting PRs.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## Security

Security is critical for a financial protocol. Please report vulnerabilities responsibly.

- 📧 **Email**: agora.ubi@tuta.io
- 🔐 See [SECURITY.md](SECURITY.md) for our security policy
- 💰 Bug bounty program coming soon

---

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- **Solana Foundation** - For the incredible blockchain platform
- **Anchor Framework** - For making Solana development accessible
- **The UBI Community** - For believing in economic freedom for all

---

<p align="center">
  <strong>For economic freedom. For everyone. Forever.</strong>
</p>

<p align="center">
  <em>"The best time to build economic alternatives was yesterday. The second best time is today."</em>
</p>

---

**No team. No ICO. No VC funding. Pure UBI.**
