import { backend } from '../model/backendProxy';
import { Crypto } from '../declarations/backend/backend.did';
import { rampErrorToString } from './error';

export const fetchOrderPrice = async (
  currency: string,
  crypto: Crypto,
): Promise<[bigint, bigint] | null> => {
  try {
    const result = await backend.calculate_order_price(currency, crypto);
    if ('Ok' in result) {
      console.log('[fetchOrderPrice] price, fee = ', result.Ok);
      return result.Ok;
    } else {
      console.error(
        'Error fetching XRC price: ',
        rampErrorToString(result.Err),
      );
      return null;
    }
  } catch (error) {
    console.error('Error fetching XRC price: ', error);
    return null;
  }
};

export const getExchangeRate = async (
  currency: string,
  crypto: string,
): Promise<number | null> => {
  try {
    const result = await backend.get_exchange_rate(currency, crypto);
    if ('Ok' in result) {
      console.log('[getExchangeRate] rate = ', result.Ok);
      return result.Ok;
    } else {
      console.error('Error fetching XRC rate: ', rampErrorToString(result.Err));
      return null;
    }
  } catch (error) {
    console.error('Error fetching XRC price: ', error);
    return null;
  }
};
