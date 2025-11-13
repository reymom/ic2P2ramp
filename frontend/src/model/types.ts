import {
  AddressType,
  BlockchainAsset,
  OrderFilter,
  PaymentProviderType,
  UserType,
} from '@/declarations/icramp_backend/icramp_backend.did';
import { RuneMetadata } from '@/declarations/bitcoin_backend/bitcoin_backend.did';

export interface TokenOption {
  name: string;
  address: string; // For runes, this can hold the serialized metadata.
  decimals: number;
  isNative: boolean;
  rateSymbol: string;
  logo: string;
  runeMetadata?: RuneMetadata; // Optional, only present for Runes.
}

type ExtractKeys<T> = T extends { [key: string]: any } ? keyof T : never;

export function candidToEnum<T extends object>(obj: T): ExtractKeys<T> {
  return Object.keys(obj)[0] as ExtractKeys<T>;
}

export type PaymentProviderTypes = ExtractKeys<PaymentProviderType>;

export type UserTypes = ExtractKeys<UserType> | 'Visitor';

export type AddressTypes = ExtractKeys<AddressType>;

export type OrderFilterTypes = ExtractKeys<OrderFilter>;

export type BlockchainTypes = ExtractKeys<BlockchainAsset>;

export const providerTypes: PaymentProviderTypes[] = [
  'PayPal',
  'Revolut',
  'Stripe',
  'Email',
  'Crypto',
];

export type revolutSchemeTypes =
  | 'UK.OBIE.IBAN'
  | 'UK.OBIE.SortCodeAccountNumber'
  | 'US.RoutingNumberAccountNumber'
  | 'US.BranchCodeAccountNumber';

export const revolutSchemes: revolutSchemeTypes[] = [
  'UK.OBIE.IBAN',
  'UK.OBIE.SortCodeAccountNumber',
  'US.RoutingNumberAccountNumber',
  'US.BranchCodeAccountNumber',
];
