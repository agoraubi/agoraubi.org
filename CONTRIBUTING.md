# Contributing to AGORA Protocol

Thank you for your interest in contributing to AGORA Protocol! 🏛️

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/agora-protocol/agora-protocol/issues)
2. If not, create a new issue with:
   - Clear, descriptive title
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Solana version, etc.)

### Suggesting Features

1. Open an issue with the `enhancement` label
2. Describe the feature and its benefits
3. Explain how it aligns with AGORA's mission

### Code Contributions

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/your-feature`
3. **Write** tests for your changes
4. **Ensure** all tests pass: `anchor test`
5. **Commit** with clear messages: `git commit -m "Add: description"`
6. **Push** to your fork: `git push origin feature/your-feature`
7. **Open** a Pull Request

### Commit Messages

Use clear, descriptive commit messages:

```
Add: new feature description
Fix: bug description
Update: what was updated
Remove: what was removed
Docs: documentation changes
Test: test additions/changes
```

## Development Setup

```bash
# Prerequisites
- Rust (latest stable)
- Solana CLI v1.17+
- Anchor v0.29+
- Node.js v18+

# Setup
git clone https://github.com/agora-protocol/agora-protocol.git
cd agora-protocol
npm install
cd contracts && anchor build
```

## Code Style

- **Rust**: Follow [Rust style guidelines](https://doc.rust-lang.org/1.0.0/style/README.html)
- **TypeScript**: Use ESLint + Prettier
- **Documentation**: Comment complex logic

## Priority Order

1. **Security** (never compromise)
2. **Correctness** (must work properly)
3. **Gas efficiency** (minimize costs)
4. **Readability** (clear code)

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Accept constructive criticism
- Focus on what's best for the community

## Questions?

- Open a GitHub issue
- Discord: Coming soon
- Twitter: [@agora_protocol](https://twitter.com/agora_protocol)

---

**Thank you for helping build economic freedom for everyone!** 🌟
