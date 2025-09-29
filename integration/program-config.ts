// DezenMart Smart Contract Integration Configuration
// Use this file to connect your frontend to the deployed smart contract

import { PublicKey } from '@solana/web3.js';

// Program Configuration
export const PROGRAM_CONFIG = {
  // Devnet deployment
  PROGRAM_ID: new PublicKey('FZVgE9vrdTHufoy197xMms8iT61q2xeeqLCAWXnUtC2C'),

  // Network configuration
  NETWORK: 'devnet',
  RPC_ENDPOINT: 'https://api.devnet.solana.com',

  // Contract constants
  ESCROW_FEE_PERCENT: 250, // 2.5%
  BASIS_POINTS: 10000,
  MAX_LOGISTICS_PROVIDERS: 10,
  MAX_PURCHASE_IDS: 1000,

  // Explorer URLs
  EXPLORER_URL: 'https://explorer.solana.com',
} as const;

// Contract Methods
export const CONTRACT_METHODS = {
  INITIALIZE: 'initialize',
  REGISTER_LOGISTICS_PROVIDER: 'registerLogisticsProvider',
  REGISTER_SELLER: 'registerSeller',
  REGISTER_BUYER: 'registerBuyer',
  CREATE_TRADE: 'createTrade',
  BUY_TRADE: 'buyTrade',
  CONFIRM_DELIVERY_AND_PURCHASE: 'confirmDeliveryAndPurchase',
  RAISE_DISPUTE: 'raiseDispute',
  RESOLVE_DISPUTE: 'resolveDispute',
  CANCEL_PURCHASE: 'cancelPurchase',
  WITHDRAW_ESCROW_FEES: 'withdrawEscrowFees',
} as const;

// Error Codes
export const CONTRACT_ERRORS = {
  UNAUTHORIZED: 6000,
  TRADE_NOT_FOUND: 6001,
  PURCHASE_NOT_FOUND: 6002,
  INSUFFICIENT_QUANTITY: 6003,
  TRADE_INACTIVE: 6004,
  INVALID_PAYMENT_AMOUNT: 6005,
  PURCHASE_ALREADY_DELIVERED: 6006,
  PURCHASE_NOT_DELIVERED: 6007,
  DISPUTE_ALREADY_EXISTS: 6008,
  NO_DISPUTE_FOUND: 6009,
  INVALID_LOGISTICS_PROVIDER: 6010,
  ALREADY_REGISTERED: 6011,
  NO_ESCROW_FEES: 6012,
} as const;

// Account Seeds
export const ACCOUNT_SEEDS = {
  GLOBAL_STATE: 'global_state',
  TRADE: 'trade',
  PURCHASE: 'purchase',
  ESCROW: 'escrow',
  SELLER: 'seller',
  BUYER: 'buyer',
  LOGISTICS_PROVIDER: 'logistics_provider',
} as const;

export default PROGRAM_CONFIG;