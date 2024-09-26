import { User } from '../declarations/backend/backend.did';
import { UserTypes } from './types';
import { userTypeToString } from './utils';

export const sessionMarginMilisec = 240_000; // 4 minuts

export const getSessionToken = (user: User | null): string | null => {
  return user && user.session && user.session.length > 0 && user.session[0]
    ? user.session[0].token
    : null;
};

export const getUserType = (user: User | null): UserTypes => {
  return user ? userTypeToString(user.user_type) : 'Visitor';
};

export const isSessionExpired = (user: User): boolean => {
  if (!user.session || user.session.length === 0) return true;

  const session = user.session[0];
  const currentTime = BigInt((Date.now() + sessionMarginMilisec) * 1_000_000);
  return session.expires_at <= currentTime;
};

const USER_SESSION_KEY = 'user_session';

export const saveUserSession = (user: User) => {
  localStorage.setItem(USER_SESSION_KEY, serializeUserSession(user));
};

export const getUserSession = (): User | null => {
  const session = localStorage.getItem(USER_SESSION_KEY);
  return session ? deserializeUserSession(session) : null;
};

export const clearUserSession = () => {
  localStorage.removeItem(USER_SESSION_KEY);
};

const CURRENCY_KEY = 'user_currency';

export const savePreferredCurrency = (currency: string) => {
  localStorage.setItem(CURRENCY_KEY, currency);
};

export const getPreferredCurrency = (): string | null => {
  return localStorage.getItem(CURRENCY_KEY) ?? null;
};

// Helper function to serialize and deserialize BigInt fields as strings
const serializeUserSession = (user: User): string => {
  return JSON.stringify(user, (_key, value) =>
    typeof value === 'bigint' ? value.toString() : value,
  );
};
const deserializeUserSession = (json: string): User => {
  return JSON.parse(json, (_key, value) =>
    typeof value === 'string' && /^\d+n?$/.test(value) ? BigInt(value) : value,
  );
};
