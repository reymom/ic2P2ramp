import { RampError } from '../declarations/backend/backend.did';

export const rampErrorToString = (error: RampError): string => {
  let errorMessage = 'An unknown error occurred';

  for (const [key, value] of Object.entries(error)) {
    if (value !== null) {
      if (typeof value === 'bigint') {
        errorMessage = `Error: ${key} - ${value.toString()}`;
      } else if (Array.isArray(value)) {
        const stringifiedValue = value.map((v) =>
          typeof v === 'bigint' ? v.toString() : v,
        );
        errorMessage = `Error: ${key} - ${JSON.stringify(stringifiedValue)}`;
      } else if (typeof value === 'object' && value !== null) {
        const stringifiedValue = Object.entries(value).reduce((acc, [k, v]) => {
          acc[k] = typeof v === 'bigint' ? v.toString() : v;
          return acc;
        }, {} as Record<string, any>);
        errorMessage = `Error: ${key} - ${JSON.stringify(stringifiedValue)}`;
      } else {
        errorMessage = `Error: ${key} - ${value}`;
      }
      break;
    } else {
      errorMessage = `Error: ${key}`;
      break;
    }
  }

  return errorMessage;
};
