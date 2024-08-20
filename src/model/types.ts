import {
  AddressType,
  Blockchain,
  OrderFilter,
  OrderStateFilter,
  PaymentProviderType,
  UserType,
} from '../declarations/backend/backend.did';

type ExtractKeys<T> = T extends { [key: string]: any } ? keyof T : never;

export function candidToEnum<T extends object>(obj: T): ExtractKeys<T> {
  return Object.keys(obj)[0] as ExtractKeys<T>;
}

export type PaymentProviderTypes = ExtractKeys<PaymentProviderType>;

export type UserTypes = ExtractKeys<UserType> | 'Visitor';

export type AddressTypes = ExtractKeys<AddressType>;

export type OrderFilterTypes = ExtractKeys<OrderFilter>;

export type OrderStateFilterTypes = ExtractKeys<OrderStateFilter>;

export type BlockchainTypes = ExtractKeys<Blockchain>;

export const providerTypes: PaymentProviderTypes[] = ['PayPal', 'Revolut'];

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
