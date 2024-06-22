import { PaymentProvider } from '../declarations/backend/backend.did';

export enum pageTypes {
  connect,
  login,
  addProvider,
  create,
  view,
}

export enum userTypes {
  onramper,
  offramper,
  visitor,
}

type ExtractKeys<T> = T extends { [key: string]: any } ? keyof T : never;

export type PaymentProviderTypes = ExtractKeys<PaymentProvider>;
