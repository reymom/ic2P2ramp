import { solana_backend as devSolanaBackend } from '@/declarations/solana_backend';

import { solana_backend_prod as prodSolanaBackend } from '@/declarations/solana_backend_prod';

const isProductionSol = process.env.FRONTEND_SOL_ENV === 'mainnet';

console.log('isProductionSol', isProductionSol);

export const solanaBackend = isProductionSol
  ? prodSolanaBackend
  : devSolanaBackend;
