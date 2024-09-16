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
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.floor(seconds % 60);
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
};
