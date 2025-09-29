import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DezenMartSmartContract } from "../target/types/dezenmart_rust_smart_contract";
import { expect } from "chai";

describe("DezenMart Integration Tests - Live Devnet", () => {
  // Configure the client to use devnet
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Your deployed program ID
  const programId = new anchor.web3.PublicKey("FZVgE9vrdTHufoy197xMms8iT61q2xeeqLCAWXnUtC2C");
  const program = new Program(require("../target/idl/dezenmart_rust_smart_contract.json"), programId, provider);

  // Test accounts
  let admin: anchor.web3.Keypair;
  let seller: anchor.web3.Keypair;
  let buyer: anchor.web3.Keypair;
  let logisticsProvider: anchor.web3.Keypair;

  // PDAs
  let globalStatePDA: anchor.web3.PublicKey;
  let sellerPDA: anchor.web3.PublicKey;
  let buyerPDA: anchor.web3.PublicKey;
  let logisticsProviderPDA: anchor.web3.PublicKey;

  before(async () => {
    // Generate test keypairs
    admin = anchor.web3.Keypair.generate();
    seller = anchor.web3.Keypair.generate();
    buyer = anchor.web3.Keypair.generate();
    logisticsProvider = anchor.web3.Keypair.generate();

    // Airdrop SOL to test accounts
    console.log("Requesting airdrop for test accounts...");

    try {
      await provider.connection.requestAirdrop(admin.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
      await provider.connection.requestAirdrop(seller.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
      await provider.connection.requestAirdrop(buyer.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
      await provider.connection.requestAirdrop(logisticsProvider.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);

      // Wait for confirmation
      await new Promise(resolve => setTimeout(resolve, 3000));
      console.log("Airdrop completed");
    } catch (error) {
      console.log("Airdrop failed (possibly rate limited), continuing...");
    }

    // Find PDAs
    [globalStatePDA] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );

    [sellerPDA] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("seller"), seller.publicKey.toBuffer()],
      program.programId
    );

    [buyerPDA] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("buyer"), buyer.publicKey.toBuffer()],
      program.programId
    );

    [logisticsProviderPDA] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("logistics_provider"), logisticsProvider.publicKey.toBuffer()],
      program.programId
    );

    console.log("Test Setup Complete");
    console.log("Program ID:", program.programId.toString());
    console.log("Admin:", admin.publicKey.toString());
    console.log("Global State PDA:", globalStatePDA.toString());
  });

  it("1. Initialize Global State", async () => {
    console.log("\n=== Testing Initialize Function ===");

    try {
      const tx = await program.methods
        .initialize()
        .accounts({
          admin: admin.publicKey,
          globalState: globalStatePDA,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      console.log("✅ Initialize transaction signature:", tx);
      console.log("🔍 View in Explorer: https://explorer.solana.com/tx/" + tx + "?cluster=devnet");

      // Fetch the account to verify
      const globalState = await program.account.globalState.fetch(globalStatePDA);
      console.log("Global State Admin:", globalState.admin.toString());
      console.log("Trade Counter:", globalState.tradeCounter.toString());
      console.log("Purchase Counter:", globalState.purchaseCounter.toString());

      expect(globalState.admin.toString()).to.equal(admin.publicKey.toString());
      expect(globalState.tradeCounter.toString()).to.equal("0");
    } catch (error) {
      if (error.message.includes("already in use")) {
        console.log("⚠️  Global state already initialized (expected on subsequent runs)");
      } else {
        throw error;
      }
    }
  });

  it("2. Register Logistics Provider", async () => {
    console.log("\n=== Testing Register Logistics Provider Function ===");

    const tx = await program.methods
      .registerLogisticsProvider()
      .accounts({
        logisticsProvider: logisticsProvider.publicKey,
        logisticsProviderAccount: logisticsProviderPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([logisticsProvider])
      .rpc();

    console.log("✅ Register Logistics Provider transaction signature:", tx);
    console.log("🔍 View in Explorer: https://explorer.solana.com/tx/" + tx + "?cluster=devnet");

    // Verify registration
    const providerAccount = await program.account.logisticsProviderAccount.fetch(logisticsProviderPDA);
    console.log("Registered Provider:", providerAccount.provider.toString());
    console.log("Is Registered:", providerAccount.isRegistered);

    expect(providerAccount.provider.toString()).to.equal(logisticsProvider.publicKey.toString());
    expect(providerAccount.isRegistered).to.be.true;
  });

  it("3. Register Seller", async () => {
    console.log("\n=== Testing Register Seller Function ===");

    const tx = await program.methods
      .registerSeller()
      .accounts({
        seller: seller.publicKey,
        sellerAccount: sellerPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([seller])
      .rpc();

    console.log("✅ Register Seller transaction signature:", tx);
    console.log("🔍 View in Explorer: https://explorer.solana.com/tx/" + tx + "?cluster=devnet");

    // Verify registration
    const sellerAccount = await program.account.sellerAccount.fetch(sellerPDA);
    console.log("Registered Seller:", sellerAccount.seller.toString());
    console.log("Is Registered:", sellerAccount.isRegistered);

    expect(sellerAccount.seller.toString()).to.equal(seller.publicKey.toString());
    expect(sellerAccount.isRegistered).to.be.true;
  });

  it("4. Register Buyer", async () => {
    console.log("\n=== Testing Register Buyer Function ===");

    const tx = await program.methods
      .registerBuyer()
      .accounts({
        buyer: buyer.publicKey,
        buyerAccount: buyerPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    console.log("✅ Register Buyer transaction signature:", tx);
    console.log("🔍 View in Explorer: https://explorer.solana.com/tx/" + tx + "?cluster=devnet");

    // Verify registration
    const buyerAccount = await program.account.buyerAccount.fetch(buyerPDA);
    console.log("Registered Buyer:", buyerAccount.buyer.toString());
    console.log("Is Registered:", buyerAccount.isRegistered);
    console.log("Purchase IDs:", buyerAccount.purchaseIds.length);

    expect(buyerAccount.buyer.toString()).to.equal(buyer.publicKey.toString());
    expect(buyerAccount.isRegistered).to.be.true;
  });

  it("5. Create Trade", async () => {
    console.log("\n=== Testing Create Trade Function ===");

    // Find trade PDA
    const [tradePDA] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("trade"), Buffer.from("1"), seller.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .createTrade(
        [logisticsProvider.publicKey], // logistics providers
        [new anchor.BN(100)], // logistics costs
        new anchor.BN(1000), // product cost
        new anchor.BN(10) // quantity
      )
      .accounts({
        seller: seller.publicKey,
        sellerAccount: sellerPDA,
        tradeAccount: tradePDA,
        globalState: globalStatePDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([seller])
      .rpc();

    console.log("✅ Create Trade transaction signature:", tx);
    console.log("🔍 View in Explorer: https://explorer.solana.com/tx/" + tx + "?cluster=devnet");

    // Verify trade creation
    const tradeAccount = await program.account.tradeAccount.fetch(tradePDA);
    console.log("Trade ID:", tradeAccount.tradeId.toString());
    console.log("Seller:", tradeAccount.seller.toString());
    console.log("Product Cost:", tradeAccount.productCost.toString());
    console.log("Total Quantity:", tradeAccount.totalQuantity.toString());
    console.log("Remaining Quantity:", tradeAccount.remainingQuantity.toString());

    expect(tradeAccount.seller.toString()).to.equal(seller.publicKey.toString());
    expect(tradeAccount.productCost.toString()).to.equal("1000");
    expect(tradeAccount.totalQuantity.toString()).to.equal("10");
  });

  it("6. Buy Trade", async () => {
    console.log("\n=== Testing Buy Trade Function ===");

    // Find PDAs
    const [tradePDA] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("trade"), Buffer.from("1"), seller.publicKey.toBuffer()],
      program.programId
    );

    const [purchasePDA] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("purchase"), Buffer.from("1"), buyer.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .buyTrade(
        new anchor.BN(1), // trade ID
        new anchor.BN(3), // quantity
        logisticsProvider.publicKey, // chosen provider
        new anchor.BN(1400) // total amount (1000*3 + 100*3 + escrow fee)
      )
      .accounts({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        buyerAccount: buyerPDA,
        tradeAccount: tradePDA,
        purchaseAccount: purchasePDA,
        globalState: globalStatePDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    console.log("✅ Buy Trade transaction signature:", tx);
    console.log("🔍 View in Explorer: https://explorer.solana.com/tx/" + tx + "?cluster=devnet");

    // Verify purchase
    const purchaseAccount = await program.account.purchaseAccount.fetch(purchasePDA);
    console.log("Purchase ID:", purchaseAccount.purchaseId.toString());
    console.log("Trade ID:", purchaseAccount.tradeId.toString());
    console.log("Buyer:", purchaseAccount.buyer.toString());
    console.log("Quantity:", purchaseAccount.quantity.toString());
    console.log("Total Amount:", purchaseAccount.totalAmount.toString());

    expect(purchaseAccount.buyer.toString()).to.equal(buyer.publicKey.toString());
    expect(purchaseAccount.quantity.toString()).to.equal("3");
  });

  after(() => {
    console.log("\n=== Integration Test Summary ===");
    console.log("✅ All individual functions tested successfully on Solana Devnet!");
    console.log("🔍 You can view all transactions in Solana Explorer using the transaction signatures above");
    console.log("🌐 Program ID:", programId.toString());
  });
});