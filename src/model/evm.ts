import { ethers } from 'ethers';
import { icP2PrampABI } from '../constants/ic2P2ramp';
import { addresses, TokenOption } from '../constants/addresses';
import { Order } from '../declarations/backend/backend.did';

export const depositInVault = async (
  selectedToken: TokenOption,
  cryptoAmount: bigint,
) => {
  if (!window.ethereum) {
    throw new Error('No crypto wallet found. Please install it.');
  }

  const provider = new ethers.BrowserProvider(window.ethereum);
  await provider.send('eth_requestAccounts', []);
  const signer = await provider.getSigner();

  const tokenAddress = selectedToken?.address;
  if (!tokenAddress) {
    throw new Error('No token selected');
  }

  const vaultContract = new ethers.Contract(tokenAddress, icP2PrampABI, signer);

  const gasEstimate = await vaultContract.depositBaseCurrency.estimateGas({
    value: cryptoAmount,
  });

  const transactionResponse = await vaultContract.depositBaseCurrency({
    value: cryptoAmount,
    gasLimit: gasEstimate,
  });

  const receipt = await transactionResponse.wait();
  if (receipt.status !== 1) {
    throw new Error('Transaction failed!');
  }

  return receipt;
};

export const withdrawFromVault = async (order: Order) => {
  if (!window.ethereum) {
    throw new Error('No crypto wallet found. Please install it.');
  }

  const provider = new ethers.BrowserProvider(window.ethereum);
  await provider.send('eth_requestAccounts', []);
  const signer = await provider.getSigner();

  const tokenAddress =
    order.crypto.token[0] ??
    addresses[Number(order.crypto.blockchain)].native[1];

  const vaultContract = new ethers.Contract(tokenAddress, icP2PrampABI, signer);

  const gasEstimate = await vaultContract.uncommitDeposit.estimateGas(
    order.offramper_address,
    ethers.ZeroAddress,
    order.crypto.amount,
  );
  const transactionResponse = await vaultContract.uncommitDeposit(
    order.offramper_address,
    ethers.ZeroAddress,
    order.crypto.amount,
    {
      gasLimit: gasEstimate,
    },
  );

  const receipt = await transactionResponse.wait();
  if (receipt.status !== 1) {
    throw new Error('Transaction failed!');
  }

  return receipt;
};
