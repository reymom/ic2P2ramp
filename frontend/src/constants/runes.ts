import dogRuneLogo from '@/assets/runes/dog-rune-logo.webp';
import puppetRuneLogo from '@/assets/runes/puppet-rune-logo.png';
import frogRuneLogo from '@/assets/runes/frog-rune-logo.webp';
import magicalBitcoinLogo from '@/assets/runes/magical-bitcoin-logo.png';

export const supportedRuneSymbols = [
  {
    runeid: '840000:3',
    symbol: '🐕',
    logo: dogRuneLogo,
    name: 'DOG•GO•TO•THE•MOON',
  },
  {
    runeid: '871680:1799',
    symbol: '🤖',
    logo: puppetRuneLogo,
    name: 'ARTIFICIAL•PUPPET',
  },
  {
    runeid: '856602:35',
    symbol: '🐸',
    logo: frogRuneLogo,
    name: 'BITCOIN•FROGS',
    divisibility: 5,
  },
  {
    runeid: '2585371:62',
    symbol: '🧙',
    logo: magicalBitcoinLogo,
    name: 'MAKE•BITCOIN•MAGICAL•AGAIN',
    divisibility: 0,
  },
];
