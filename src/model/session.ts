import { User } from '../declarations/backend/backend.did';

const USER_SESSION_KEY = 'user_session';

type UserSession = {
  sessionToken: string;
  user: User;
};

export const saveUserSession = (userSession: UserSession) => {
  localStorage.setItem(USER_SESSION_KEY, serializeUserSession(userSession));
};

export const getUserSession = (): UserSession | null => {
  const session = localStorage.getItem(USER_SESSION_KEY);
  return session ? deserializeUserSession(session) : null;
};

export const clearUserSession = () => {
  localStorage.removeItem(USER_SESSION_KEY);
};

// Helper function to serialize and deserialize BigInt fields as strings
const serializeUserSession = (userSession: UserSession): string => {
  return JSON.stringify(userSession, (key, value) =>
    typeof value === 'bigint' ? value.toString() : value,
  );
};
const deserializeUserSession = (json: string): UserSession => {
  return JSON.parse(json, (key, value) =>
    typeof value === 'string' && /^\d+n?$/.test(value) ? BigInt(value) : value,
  );
};
