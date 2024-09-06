const SESSION_TOKEN_KEY = 'session_token';

// Save session token to localStorage
export const saveSessionToken = (token: string) => {
  localStorage.setItem(SESSION_TOKEN_KEY, token);
};

// Get session token from localStorage
export const getSessionToken = (): string | null => {
  return localStorage.getItem(SESSION_TOKEN_KEY);
};

// Remove session token from localStorage
export const clearSessionToken = () => {
  localStorage.removeItem(SESSION_TOKEN_KEY);
};
