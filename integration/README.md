# DezenMart Smart Contract Frontend Integration

## 🔗 Quick Start

### 1. Install Dependencies
```bash
npm install @solana/web3.js @solana/spl-token
```

### 2. Import Configuration
```typescript
import { PROGRAM_CONFIG } from './program-config';
import { DezenMartClient } from './client-utils';
```

### 3. Initialize Client
```typescript
const client = new DezenMartClient();
```

## 📊 Program Information

- **Program ID**: `FZVgE9vrdTHufoy197xMms8iT61q2xeeqLCAWXnUtC2C` ✅ **DEPLOYED**
- **Network**: Solana Devnet ✅ **LIVE**
- **Explorer**: [View on Solana Explorer](https://explorer.solana.com/address/FZVgE9vrdTHufoy197xMms8iT61q2xeeqLCAWXnUtC2C?cluster=devnet)
- **Deployment TX**: `2QarpKCTfkqE4672WnvAAHKojavHZoBUs62QiZRfLN5mby7Egitq1678WCG77ZaXLcxZfcuztevt42ExVRKo3yWa`

## 🛠 Integration Examples

### Create Trade
```typescript
// Get required PDAs
const [tradePDA] = await client.getTradePDA(tradeId, sellerPublicKey);
const [sellerPDA] = await client.getSellerPDA(sellerPublicKey);

// Validate parameters
const validation = client.validateTradeParameters({
  productCost: 1000,
  logisticsCosts: [100, 200],
  logisticsProviders: [provider1, provider2],
  totalQuantity: 50,
});

if (!validation.valid) {
  console.error('Validation errors:', validation.errors);
  return;
}
```

### Calculate Fees
```typescript
const productCost = 1000;
const logisticsCost = 200;
const quantity = 5;

const escrowFee = client.calculateEscrowFee(productCost * quantity);
const totalCost = (productCost + logisticsCost) * quantity + escrowFee;

console.log(`Total cost: ${totalCost} tokens (including ${escrowFee} escrow fee)`);
```

### Get Account PDAs
```typescript
// Global state
const [globalStatePDA] = await client.getGlobalStatePDA();

// User accounts
const [buyerPDA] = await client.getBuyerPDA(buyerPublicKey);
const [sellerPDA] = await client.getSellerPDA(sellerPublicKey);
const [providerPDA] = await client.getLogisticsProviderPDA(providerPublicKey);

// Trade and purchase accounts
const [tradePDA] = await client.getTradePDA(tradeId, sellerPublicKey);
const [purchasePDA] = await client.getPurchasePDA(purchaseId, buyerPublicKey);

// Escrow account
const [escrowPDA] = await client.getEscrowTokenPDA(tokenMint);
```

## 🏛 Available Methods

| Method | Description | Access |
|--------|-------------|---------|
| `initialize` | Initialize global state | Admin only |
| `registerLogisticsProvider` | Register as logistics provider | Public |
| `registerSeller` | Register as seller | Admin approval |
| `registerBuyer` | Register as buyer | Public |
| `createTrade` | Create trade listing | Registered sellers |
| `buyTrade` | Purchase from trade | Registered buyers |
| `confirmDeliveryAndPurchase` | Confirm delivery | Buyer |
| `raiseDispute` | Raise dispute | Buyer |
| `resolveDispute` | Resolve dispute | Admin only |
| `cancelPurchase` | Cancel and refund | Buyer |
| `withdrawEscrowFees` | Withdraw fees | Admin only |

## 🚨 Error Handling

```typescript
import { CONTRACT_ERRORS } from './program-config';

// Handle specific contract errors
switch (errorCode) {
  case CONTRACT_ERRORS.TRADE_NOT_FOUND:
    console.error('Trade not found');
    break;
  case CONTRACT_ERRORS.INSUFFICIENT_QUANTITY:
    console.error('Not enough quantity available');
    break;
  case CONTRACT_ERRORS.UNAUTHORIZED:
    console.error('User not authorized for this action');
    break;
  // ... handle other errors
}
```

## 📱 Next Steps

1. Copy `program-config.ts` and `client-utils.ts` to your frontend project
2. Install Solana dependencies
3. Initialize the client in your application
4. Build transaction instructions using the helper methods
5. Test with the deployed devnet contract

For complete deployment instructions, see [DEPLOYMENT.md](../DEPLOYMENT.md).