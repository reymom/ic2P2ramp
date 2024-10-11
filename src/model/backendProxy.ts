import {
  createActor as createDevActor,
  backend as devBackend,
} from '../declarations/backend';
import {
  createActor as createProdActor,
  backend_prod as prodBackend,
} from '../declarations/backend_prod';

const isProduction = process.env.FRONTEND_EVM_ENV === 'production';

export const backend = isProduction ? prodBackend : devBackend;
export const createActor = isProduction ? createProdActor : createDevActor;
