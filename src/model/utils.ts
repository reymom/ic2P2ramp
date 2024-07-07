import {
  OrderFilter,
  OrderStateFilter,
  PaymentProviderType,
  UserType,
} from '../declarations/backend/backend.did';
import {
  OrderFilterTypes,
  OrderStateFilterTypes,
  UserTypes,
  PaymentProviderTypes,
  candidToEnum,
} from './types';

// Payment Providers
export const paymentProviderTypeToString = (
  providerType: PaymentProviderType,
): PaymentProviderTypes => {
  if ('PayPal' in providerType) return 'PayPal';
  if ('Revolut' in providerType) return 'Revolut';
  throw new Error('Unknown payment provider');
};

export const stringToPaymentProviderType = (
  providerType: PaymentProviderTypes,
): PaymentProviderType => {
  return { [providerType]: null } as PaymentProviderType;
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

// -------------
// Order Filters
// -------------
export const filterToFilterType = (filter: OrderFilter): OrderFilterTypes => {
  return candidToEnum(filter);
};

export const stringToOrderFilter = (key: string, value: any): OrderFilter => {
  return { [key]: value } as OrderFilter;
};

// -------------------
// Order State Filters
// -------------------
export const filterStateToFilterStateType = (
  stateFilter: OrderStateFilter,
): OrderStateFilterTypes => {
  return candidToEnum(stateFilter);
};

export const stringToOrderStateFilter = (
  key: OrderStateFilterTypes,
): OrderStateFilter => {
  return { [key]: null } as OrderStateFilter;
};
