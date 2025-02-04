export const truncate = (
  str: string,
  frontChars: number,
  backChars: number,
) => {
  if (str.length <= frontChars + backChars) {
    return str;
  }
  return str.slice(0, frontChars) + '...' + str.slice(-backChars);
};

export const validatePassword = (password: string): string | null => {
  const minLength = 8;
  const hasNumber = /\d/;
  const hasUppercase = /[A-Z]/;

  if (password.length < minLength) {
    return `Password must be at least ${minLength} characters long.`;
  }
  if (!hasNumber.test(password)) {
    return 'Password must contain at least one number.';
  }
  if (!hasUppercase.test(password)) {
    return 'Password must contain at least one uppercase letter.';
  }
  return null;
};

export const formatTimeLeft = (seconds: number) => {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainingSeconds = Math.floor(seconds % 60);
  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${remainingSeconds
      .toString()
      .padStart(2, '0')}`;
  } else {
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  }
};

export const formatCryptoUnits = (amount: number): string => {
  if (amount === 0) return '0.00';
  if (amount < 0.00001) return amount.toFixed(7);
  if (amount < 0.0001) return amount.toFixed(6);
  if (amount < 0.001) return amount.toFixed(5);
  if (amount < 1) return amount.toFixed(4);
  if (amount < 1000) return amount.toFixed(2);
  if (amount > 1000000000) return amount.toExponential(4);
  return amount.toFixed(2);
};

export const formatPrice = (centsAmount: number): string => {
  return (centsAmount / 100).toFixed(2);
};
