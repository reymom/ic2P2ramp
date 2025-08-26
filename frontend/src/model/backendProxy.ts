import {
  createActor as createDevActor,
  icramp_backend as devBackend,
} from '@/declarations/icramp_backend';
import {
  createActor as createProdActor,
  icramp_backend_prod as prodBackend,
} from '@/declarations/icramp_backend_prod';

const isProduction = process.env.FRONTEND_EVM_ENV === 'production';

console.log('isProduction', isProduction);

export const backend = isProduction ? prodBackend : devBackend;
export const createActor = isProduction ? createProdActor : createDevActor;
