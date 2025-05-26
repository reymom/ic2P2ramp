import {
  bitcoin_backend as devBitcoinBackend,
  createActor as createDevBitcoinActor,
} from '@/declarations/bitcoin_backend';

import {
  bitcoin_backend_prod as prodBitcoinBackend,
  createActor as createProdBitcoinActor,
} from '@/declarations/bitcoin_backend_prod';

const isProductionBTC = process.env.FRONTEND_BTC_ENV === 'mainnet';

console.log('isProductionBTC', isProductionBTC);

export const bitcoin_backend = isProductionBTC
  ? prodBitcoinBackend
  : devBitcoinBackend;
export const createBitcoinActor = isProductionBTC
  ? createProdBitcoinActor
  : createDevBitcoinActor;
