import {
  faDollarSign,
  faEuroSign,
  faPoundSign,
} from '@fortawesome/free-solid-svg-icons';
import { IconDefinition } from '@fortawesome/fontawesome-svg-core';

export const CURRENCY_ICON_MAP: { [symbol: string]: IconDefinition } = {
  USD: faDollarSign,
  EUR: faEuroSign,
  GBP: faPoundSign,
};
