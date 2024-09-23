import { backend } from '../declarations/backend';
import { Crypto } from '../declarations/backend/backend.did';
import { rampErrorToString } from './error';

export const fetchOrderPrice = async (
  currency: string,
  crypto: Crypto,
): Promise<[bigint, bigint]> => {
  try {
    const result = await backend.calculate_order_price(currency, crypto);
    if ('Ok' in result) {
      console.log('[fetchOrderPrice] price, fee', result.Ok);
      return result.Ok;
    } else {
      throw new Error(rampErrorToString(result.Err));
    }
  } catch (error) {
    console.error('Error fetching XRC price:', error);
    throw new Error('Could not fetch XRC price.');
  }
};

export const getExchangeRate = async (
  currency: string,
  crypto: string,
): Promise<number> => {
  try {
    const result = await backend.get_exchange_rate(currency, crypto);
    if ('Ok' in result) {
      console.log('[getExchangeRate] rate = ', result.Ok);
      return result.Ok;
    } else {
      throw new Error(rampErrorToString(result.Err));
    }
  } catch (error) {
    console.error('Error fetching XRC price:', error);
    throw new Error('Could not fetch XRC price');
  }
};
