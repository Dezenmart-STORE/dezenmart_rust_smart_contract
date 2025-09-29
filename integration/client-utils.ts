// DezenMart Smart Contract Client Utilities
// Helper functions for frontend integration

import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PROGRAM_CONFIG, ACCOUNT_SEEDS } from './program-config';

export class DezenMartClient {
  private connection: Connection;
  private programId: PublicKey;

  constructor(rpcEndpoint?: string) {
    this.connection = new Connection(rpcEndpoint || PROGRAM_CONFIG.RPC_ENDPOINT, 'confirmed');
    this.programId = PROGRAM_CONFIG.PROGRAM_ID;
  }

  // Get Program Derived Address for different account types
  async findProgramAddress(seeds: (string | Buffer | Uint8Array)[]): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(seeds, this.programId);
  }

  // Get Global State PDA
  async getGlobalStatePDA(): Promise<[PublicKey, number]> {
    return await this.findProgramAddress([Buffer.from(ACCOUNT_SEEDS.GLOBAL_STATE)]);
  }

  // Get Trade Account PDA
  async getTradePDA(tradeId: number, seller: PublicKey): Promise<[PublicKey, number]> {
    return await this.findProgramAddress([
      Buffer.from(ACCOUNT_SEEDS.TRADE),
      Buffer.from(tradeId.toString()),
      seller.toBuffer(),
    ]);
  }

  // Get Purchase Account PDA
  async getPurchasePDA(purchaseId: number, buyer: PublicKey): Promise<[PublicKey, number]> {
    return await this.findProgramAddress([
      Buffer.from(ACCOUNT_SEEDS.PURCHASE),
      Buffer.from(purchaseId.toString()),
      buyer.toBuffer(),
    ]);
  }

  // Get Seller Account PDA
  async getSellerPDA(seller: PublicKey): Promise<[PublicKey, number]> {
    return await this.findProgramAddress([
      Buffer.from(ACCOUNT_SEEDS.SELLER),
      seller.toBuffer(),
    ]);
  }

  // Get Buyer Account PDA
  async getBuyerPDA(buyer: PublicKey): Promise<[PublicKey, number]> {
    return await this.findProgramAddress([
      Buffer.from(ACCOUNT_SEEDS.BUYER),
      buyer.toBuffer(),
    ]);
  }

  // Get Logistics Provider Account PDA
  async getLogisticsProviderPDA(provider: PublicKey): Promise<[PublicKey, number]> {
    return await this.findProgramAddress([
      Buffer.from(ACCOUNT_SEEDS.LOGISTICS_PROVIDER),
      provider.toBuffer(),
    ]);
  }

  // Get Escrow Token Account PDA
  async getEscrowTokenPDA(tokenMint: PublicKey): Promise<[PublicKey, number]> {
    return await this.findProgramAddress([
      Buffer.from(ACCOUNT_SEEDS.ESCROW),
      tokenMint.toBuffer(),
    ]);
  }

  // Helper to get account info
  async getAccountInfo(publicKey: PublicKey) {
    return await this.connection.getAccountInfo(publicKey);
  }

  // Helper to send transaction
  async sendTransaction(transaction: Transaction, signers: any[]) {
    return await this.connection.sendTransaction(transaction, signers);
  }

  // Get explorer URL for transaction/account
  getExplorerUrl(signature: string, type: 'tx' | 'address' = 'tx'): string {
    const baseUrl = PROGRAM_CONFIG.EXPLORER_URL;
    const cluster = PROGRAM_CONFIG.NETWORK === 'devnet' ? '?cluster=devnet' : '';
    return `${baseUrl}/${type}/${signature}${cluster}`;
  }

  // Calculate escrow fee
  calculateEscrowFee(amount: number): number {
    return Math.floor((amount * PROGRAM_CONFIG.ESCROW_FEE_PERCENT) / PROGRAM_CONFIG.BASIS_POINTS);
  }

  // Validate trade parameters
  validateTradeParameters(params: {
    productCost: number;
    logisticsCosts: number[];
    logisticsProviders: PublicKey[];
    totalQuantity: number;
  }): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (params.productCost <= 0) {
      errors.push('Product cost must be greater than 0');
    }

    if (params.totalQuantity <= 0) {
      errors.push('Total quantity must be greater than 0');
    }

    if (params.logisticsProviders.length !== params.logisticsCosts.length) {
      errors.push('Logistics providers and costs arrays must have the same length');
    }

    if (params.logisticsProviders.length > PROGRAM_CONFIG.MAX_LOGISTICS_PROVIDERS) {
      errors.push(`Maximum ${PROGRAM_CONFIG.MAX_LOGISTICS_PROVIDERS} logistics providers allowed`);
    }

    params.logisticsCosts.forEach((cost, index) => {
      if (cost <= 0) {
        errors.push(`Logistics cost at index ${index} must be greater than 0`);
      }
    });

    return {
      valid: errors.length === 0,
      errors,
    };
  }
}

// Export singleton instance
export const dezenMartClient = new DezenMartClient();

export default DezenMartClient;