import { NetworkIds } from '@/constants/networks';
import {
  BlockchainAsset,
  PaymentProvider,
  PaymentProviderType,
  UserType,
} from '@/declarations/icramp_backend/icramp_backend.did';
import {
  UserTypes,
  PaymentProviderTypes,
  BlockchainTypes,
} from '@/model/types';

// Blockchain
export const blockchainAssetToBlockchainType = (
  asset: BlockchainAsset,
): BlockchainTypes => {
  if ('EVM' in asset) return 'EVM';
  if ('ICP' in asset) return 'ICP';
  if ('Solana' in asset) return 'Solana';
  if ('Bitcoin' in asset) return 'Bitcoin';
  throw new Error('Unknown blockchain');
};

export const blockchainAssetToChain = (asset: BlockchainAsset) => {
  if ('EVM' in asset)
    return (
      Object.values(NetworkIds).find((n) => n.id === Number(asset.EVM.chain_id))
        ?.name ?? 'EVM'
    );
  if ('ICP' in asset) return 'ICP';
  if ('Solana' in asset) return 'Solana';
  if ('Bitcoin' in asset) return 'Bitcoin';
  throw new Error('Unknown blockchain');
};

// Payment Providers
export const paymentProviderTypeToString = (
  providerType: PaymentProviderType,
): PaymentProviderTypes => {
  if ('PayPal' in providerType) return 'PayPal';
  if ('Revolut' in providerType) return 'Revolut';
  if ('Stripe' in providerType) return 'Stripe';
  if ('Email' in providerType) return 'Email';
  if ('Crypto' in providerType) return 'Crypto';
  throw new Error('Unknown payment provider');
};

export const providerToProviderType = (
  provider: PaymentProvider,
): PaymentProviderType => {
  if ('PayPal' in provider) return { PayPal: null };
  if ('Revolut' in provider) return { Revolut: null };
  if ('Stripe' in provider) return { Stripe: null };
  if ('Email' in provider) return { Email: null };
  if ('Crypto' in provider) return { Crypto: null };
  throw new Error('Unkown provider type');
};

// -----
// Users
// -----
export const userTypeToString = (userType: UserType): UserTypes => {
  if ('Offramper' in userType) return 'Offramper';
  if ('Onramper' in userType) return 'Onramper';
  throw new Error('Unknown user type');
};

export const stringToUserType = (userType: UserTypes): UserType => {
  switch (userType) {
    case 'Visitor':
      throw new Error('Unknown user type');
    default:
      return { [userType]: null } as UserType;
  }
};
