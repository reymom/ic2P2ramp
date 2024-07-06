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
