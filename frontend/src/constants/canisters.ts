let _backendId: string | null = null;

export function getBackendCanisterId(): string {
  if (_backendId) return _backendId;

  let id = process.env.CANISTER_ID_ICRAMP_BACKEND;
  if (process.env.FRONTEND_BTC_ENV === 'mainnet') {
    id = process.env.CANISTER_ID_ICRAMP_BACKEND_PROD;
  }
  if (!id) throw new Error('Backend Canister ID not in env file');

  _backendId = id;
  return _backendId;
}
