import { TokenOption } from '../model/types';

import icpLogo from '../assets/blockchains/icp-logo.svg';
import openChatLogo from '../assets/blockchains/openchat-logo.svg';
import ckBTCLogo from '../assets/blockchains/ckBTC-logo.svg';
import origynLogo from '../assets/blockchains/origyn-logo.png';

if (!process.env.CANISTER_ID_BACKEND) {
  console.error('Backend canister id not defined');
}

export const ICP_TOKENS: TokenOption[] = [
  {
    name: 'ICP',
    address: 'ryjl3-tyaaa-aaaaa-aaaba-cai',
    decimals: 8,
    isNative: true,
    rateSymbol: 'ICP',
    logo: icpLogo,
  },
  {
    name: 'ckBTC',
    address:
      process.env.FRONTEND_ICP_ENV === 'production'
        ? 'mxzaz-hqaaa-aaaar-qaada-cai'
        : 'mc6ru-gyaaa-aaaar-qaaaq-cai',
    decimals: 8,
    isNative: false,
    rateSymbol: 'BTC',
    logo: ckBTCLogo,
  },
  {
    name: 'CHAT',
    address:
      process.env.FRONTEND_ICP_ENV === 'production'
        ? '2ouva-viaaa-aaaaq-aaamq-cai'
        : '',
    decimals: 8,
    isNative: false,
    rateSymbol: 'CHAT',
    logo: openChatLogo,
  },
  {
    name: 'OGY',
    address:
      process.env.FRONTEND_ICP_ENV === 'production'
        ? 'lkwrt-vyaaa-aaaaq-aadhq-cai'
        : '',
    decimals: 8,
    isNative: false,
    rateSymbol: 'OGY',
    logo: origynLogo,
  },
].filter((token) => token.address !== '');
