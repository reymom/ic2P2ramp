import { RampError } from '../declarations/backend/backend.did';

export const rampErrorToString = (error: RampError): string => {
  let errorMessage = 'An unknown error occurred';

  for (const [key, value] of Object.entries(error)) {
    if (value !== null && typeof value === 'object') {
      // Handle the second-level variant
      for (const [nestedKey, nestedValue] of Object.entries(value)) {
        if (nestedValue !== null) {
          if (typeof nestedValue === 'bigint') {
            errorMessage = `${key}: ${nestedKey} - ${nestedValue.toString()}`;
          } else if (Array.isArray(nestedValue)) {
            const stringifiedNestedValue = nestedValue.map((v) =>
              typeof v === 'bigint' ? v.toString() : v,
            );
            errorMessage = `Error: ${key} - ${nestedKey} - ${JSON.stringify(
              stringifiedNestedValue,
            )}`;
          } else if (typeof nestedValue === 'object') {
            const stringifiedNestedValue = Object.entries(nestedValue).reduce(
              (acc, [k, v]) => {
                acc[k] = typeof v === 'bigint' ? v.toString() : v;
                return acc;
              },
              {} as Record<string, any>,
            );
            errorMessage = `Error: ${key} - ${nestedKey} - ${JSON.stringify(
              stringifiedNestedValue,
            )}`;
          } else {
            errorMessage = `${key}: ${nestedKey} - ${nestedValue}`;
          }
        } else {
          errorMessage = `${key}: ${nestedKey}`;
        }
        break;
      }
    } else {
      errorMessage = `${key}`;
    }
    break;
  }

  return errorMessage;
};

// export const rampErrorToString = (error: RampError): string => {
//   let errorMessage = 'An unknown error occurred';

//   for (const [key, value] of Object.entries(error)) {
//     if (value !== null) {
//       if (typeof value === 'bigint') {
//         errorMessage = `Error: ${key} - ${value.toString()}`;
//       } else if (Array.isArray(value)) {
//         const stringifiedValue = value.map((v) =>
//           typeof v === 'bigint' ? v.toString() : v,
//         );
//         errorMessage = `Error: ${key} - ${JSON.stringify(stringifiedValue)}`;
//       } else if (typeof value === 'object' && value !== null) {
//         const stringifiedValue = Object.entries(value).reduce((acc, [k, v]) => {
//           acc[k] = typeof v === 'bigint' ? v.toString() : v;
//           return acc;
//         }, {} as Record<string, any>);
//         errorMessage = `Error: ${key} - ${JSON.stringify(stringifiedValue)}`;
//       } else {
//         errorMessage = `Error: ${key} - ${value}`;
//       }
//       break;
//     } else {
//       errorMessage = `Error: ${key}`;
//       break;
//     }
//   }

//   return errorMessage;
// };

export const isUserNotFoundError = (error: RampError): boolean => {
  if ('UserError' in error && 'UserNotFound' in error.UserError) {
    return true;
  }
  return false;
};

export const isInvalidPasswordError = (error: RampError): boolean => {
  if ('UserError' in error && 'InvalidPassword' in error.UserError) {
    return true;
  }
  return false;
};

export const isUnauthorizedPrincipalError = (error: RampError): boolean => {
  if ('UserError' in error && 'UnauthorizedPrincipal' in error.UserError) {
    return true;
  }
  return false;
};
