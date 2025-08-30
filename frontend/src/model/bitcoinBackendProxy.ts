import { bitcoin_backend as devBitcoinBackend } from '@/declarations/bitcoin_backend';

import { bitcoin_backend_prod as prodBitcoinBackend } from '@/declarations/bitcoin_backend_prod';

const isProductionBTC = process.env.FRONTEND_BTC_ENV === 'mainnet';

console.log('isProductionBTC', isProductionBTC);

export const bitcoinBackend = isProductionBTC
  ? prodBitcoinBackend
  : devBitcoinBackend;
