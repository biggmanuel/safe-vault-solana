import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SafeVault } from "../target/types/safe_vault";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { assert } from "chai";

describe("safe_vault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SafeVault as Program<SafeVault>;
  const user = (provider.wallet as anchor.Wallet).payer;

  // Global variables we'll need across tests
  let mint: PublicKey;
  let vaultState: PublicKey;
  let vaultTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let userStats: PublicKey;

  // PDAs
  const [vaultStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_state")],
    program.programId
  );
  
  const [vaultTokenPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_tokens")],
    program.programId
  );

  const [userStatsPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("user-stats"), user.publicKey.toBuffer()],
    program.programId
  );

  it("Is initialized!", async () => {
    // 1. Create a fake token mint (Mock USDC)
    mint = await createMint(
      provider.connection,
      user,
      user.publicKey,
      null,
      6
    );

    console.log("Mint Created:", mint.toBase58());

    // 2. Initialize the Vault
    const tx = await program.methods
      .initialize()
      .accounts({
        vaultAccount: vaultStatePda,
        vaultTokenAccount: vaultTokenPda,
        mint: mint,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    
    console.log("Vault Initialized. Tx:", tx);
  });

  it("User deposits Collateral", async () => {
    // 1. Get user's token account and mint them some tokens
    const userAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user,
      mint,
      user.publicKey
    );
    userTokenAccount = userAta.address;

    // Mint 1000 tokens to user
    await mintTo(
      provider.connection,
      user,
      mint,
      userTokenAccount,
      user,
      1000
    );

    // 2. Deposit 100 tokens into the vault
    const depositAmount = new anchor.BN(100);
    
    await program.methods
      .deposit(depositAmount)
      .accounts({
        vaultAccount: vaultStatePda,
        userAccount: userStatsPda,
        userTokenAccount: userTokenAccount,
        vaultTokenAccount: vaultTokenPda,
        user: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Deposited 100 tokens.");
    
    // Verify on-chain state
    const userAccountState = await program.account.userStats.fetch(userStatsPda);
    assert.ok(userAccountState.collateralAmount.toNumber() === 100);
  });

  it("User borrows safely (LTV < 50%)", async () => {
    // Collateral = 100, Price = 100, Value = 10,000.
    // Max Loan = 5,000.
    // We borrow 40.
    const borrowAmount = new anchor.BN(40);

    await program.methods
      .borrow(borrowAmount)
      .accounts({
        vaultAccount: vaultStatePda,
        userAccount: userStatsPda,
        vaultTokenAccount: vaultTokenPda,
        userTokenAccount: userTokenAccount,
        user: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Borrowed 40 tokens.");
    
    const userAccountState = await program.account.userStats.fetch(userStatsPda);
    assert.ok(userAccountState.borrowedAmount.toNumber() === 40);
  });

  it("Fails when borrowing too much (LTV > 50%)", async () => {
    // Try to borrow 9000 (Max is 5000)
    const borrowAmount = new anchor.BN(9000);

    try {
      await program.methods
        .borrow(borrowAmount)
        .accounts({
          vaultAccount: vaultStatePda,
          userAccount: userStatsPda,
          vaultTokenAccount: vaultTokenPda,
          userTokenAccount: userTokenAccount,
          user: user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
        
      assert.fail("Should have failed due to InsufficientCollateral");
    } catch (err) {
      assert.ok(err.message.includes("Insufficient collateral"), "Error message didn't match");
      console.log("Success: The contract correctly rejected the bad loan.");
    }
  });
});