# SafeVault 🛡️

**A production-grade DeFi lending primitive on Solana built with Anchor.**

SafeVault is an over-collateralized lending protocol that allows users to deposit assets and borrow against them, enforcing strict Loan-to-Value (LTV) ratios to ensure protocol solvency. 

*Built to demonstrate secure program architecture and rigorous integration testing.*

## 🚀 Key Features

* **Over-Collateralized Lending:** Enforces a maximum 50% LTV ratio. Users cannot borrow more than 50% of their collateral value.
* **Atomic Solvency Checks:** Uses `require!` macros to validate health factors *before* any token transfer occurs, preventing bad debt creation.
* **PDA Asset Management:** Utilizes Program Derived Addresses (PDAs) for secure vault custody, ensuring only the program can sign for withdrawals.
* **Optimized Compute:** Structured to minimize instruction count and cross-program invocation overhead.

## 🛠️ Tech Stack

* **Language:** Rust (Anchor Framework v0.30+)
* **Testing:** TypeScript (Mocha/Chai) & Local Validator
* **Network:** Solana Localnet / Devnet

## 🧪 Testing Strategy

This repository includes a comprehensive Integration Test suite (`tests/safe_vault.ts`) designed to validate security invariants.

### Covered Scenarios:
1.  **✅ Initialization:** Verifies correct PDA generation for Vault state and Token vaults.
2.  **✅ Deposit:** Ensures user collateral is correctly tracked in the `UserStats` account.
3.  **✅ Safe Borrow:** Validates that users can borrow within safe limits.
4.  **🛡️ Insolvency Protection (Security Check):** Explicitly attempts to borrow *more* than the allowed LTV. The test asserts that the program correctly rejects the transaction with an `InsufficientCollateral` error.

### How to Run Tests
```bash
# 1. Install dependencies
yarn install

# 2. Build the program
anchor build

# 3. Run the test suite
anchor test

📂 Project Structure
programs/safe_vault/src/lib.rs - Core Logic. Contains the Deposit and Borrow instructions with security assertions.

tests/safe_vault.ts - Integration Tests. TypeScript tests verifying success and failure modes.

🔐 Security Considerations
Arithmetic Safety: All math operations utilize Rust's overflow checks (or checked math where explicit) to prevent integer overflows.

Access Control: Signer checks are enforced on all user interactions; Vault PDAs are validated via canonical bump seeds.

Oracle Integration: Current implementation uses a mock price feed ($100). Production readiness would involve integrating Switchboard or Pyth for real-time price updates.

👤 Author
Bigg Manuel 