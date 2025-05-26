import dogRuneLogo from '@/assets/runes/dog-rune-logo.webp';
import puppetRuneLogo from '@/assets/runes/puppet-rune-logo.png';
import frogRuneLogo from '@/assets/runes/frog-rune-logo.webp';
import magicalBitcoinLogo from '@/assets/runes/magical-bitcoin-logo.png';
import uncommonGoodsLogo from '@/assets/runes/uncommon-goods-logo.png';

interface RuneIds {
  runeId: string;
  symbol: string;
  logo: string;
  name: string;
}

const mainnetRuneIds: RuneIds[] = [
  {
    runeId: '840000:3',
    symbol: '🐕',
    logo: dogRuneLogo,
    name: 'DOG•GO•TO•THE•MOON',
  },
  {
    runeId: '1:0',
    symbol: '⧉',
    logo: uncommonGoodsLogo,
    name: 'UNCOMMON•GOODS',
  },
  {
    runeId: '871680:1799',
    symbol: '🤖',
    logo: puppetRuneLogo,
    name: 'ARTIFICIAL•PUPPET',
  },
  {
    runeId: '856602:35',
    symbol: '🐸',
    logo: frogRuneLogo,
    name: 'BITCOIN•FROGS',
  },
  {
    runeId: '868973:1169',
    symbol: '🧙',
    logo: magicalBitcoinLogo,
    name: 'MAKE•BITCOIN•MAGICAL•AGAIN',
  },
];

const testRuneIds: RuneIds[] = [
  {
    runeId: '66593:594',
    symbol: '🐕',
    logo: dogRuneLogo,
    name: 'DOG•GO•TO•THE•MOON',
  },
  {
    runeId: '73393:191',
    symbol: '⧉',
    logo: uncommonGoodsLogo,
    name: 'UNCOMMON•GOODS',
  },
];

export const supportedRuneIds =
  process.env.FRONTEND_BTC_ENV === 'mainnet' ? mainnetRuneIds : testRuneIds;
