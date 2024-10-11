import { ethers } from 'ethers';

import { backend } from '../model/backendProxy';
import { TransactionAction } from '../declarations/backend/backend.did';
import { icP2PrampABI } from '../constants/ic2P2ramp';
import { getVaultAddress } from '../constants/evm_tokens';
import { TokenOption } from '../model/types';

export const depositInVault = async (
  chainId: number,
  selectedToken: TokenOption,
  cryptoAmount: bigint,
) => {
  if (!window.ethereum)
    throw new Error('No crypto wallet found. Please install it.');

  const provider = new ethers.BrowserProvider(window.ethereum);
  await provider.send('eth_requestAccounts', []);
  const signer = await provider.getSigner();

  const vaultContract = new ethers.Contract(
    getVaultAddress(chainId),
    icP2PrampABI,
    signer,
  );

  try {
    let transactionResponse;
    if (selectedToken.isNative) {
      const gasEstimate = await vaultContract.depositBaseCurrency.estimateGas({
        value: cryptoAmount,
      });

      transactionResponse = await vaultContract.depositBaseCurrency({
        value: cryptoAmount,
        gasLimit: gasEstimate,
      });
    } else if (selectedToken.address !== '') {
      // approve the vault contract to spend the tokens
      const tokenContract = new ethers.Contract(
        selectedToken.address,
        [
          'function approve(address spender, uint256 amount) external returns (bool)',
        ],
        signer,
      );

      const approveTx = await tokenContract.approve(
        getVaultAddress(chainId),
        cryptoAmount,
      );
      await approveTx.wait();

      // make deposit
      const gasEstimate = await vaultContract.depositToken.estimateGas(
        selectedToken.address,
        cryptoAmount,
      );

      transactionResponse = await vaultContract.depositToken(
        selectedToken.address,
        cryptoAmount,
        {
          gasLimit: gasEstimate,
        },
      );
    } else {
      throw new Error('No token selected');
    }

    const receipt = await transactionResponse.wait();
    if (receipt.status !== 1) {
      throw new Error('Transaction failed on-chain.');
    }
    return receipt;
  } catch (error: any) {
    throw new Error(handleWeb3Error(error));
  }
};

export const estimateGasAndGasPrice = async (
  chainId: number,
  method: TransactionAction,
  defaultGas: bigint,
  days: number = 7,
): Promise<[bigint, bigint]> => {
  const blockTimeInSeconds = 12; // Approximate block time
  const blocksPerDay = (24 * 60 * 60) / blockTimeInSeconds;
  const maxBlocksInPast = BigInt(Math.ceil(blocksPerDay * days));

  const response = await backend.get_average_gas_prices(
    BigInt(chainId),
    maxBlocksInPast,
    method,
  );

  if ('Ok' in response && response.Ok.length > 0) {
    return response.Ok[0]!;
  } else if ('Err' in response) {
    console.error('[estimateGasAndGasPrice] error = ', response.Err);
  }

  return [defaultGas, BigInt(0)];
};

export const estimateOrderFees = async (
  chainId: bigint,
  cryptoAmount: bigint,
  token: [] | [string],
  gasForCommit: bigint,
  gasForRelease: bigint,
): Promise<bigint> => {
  try {
    const estimateOrderFees = await backend.calculate_order_evm_fees(
      chainId,
      cryptoAmount,
      token,
      gasForCommit,
      gasForRelease,
    );

    if ('Ok' in estimateOrderFees) {
      return estimateOrderFees.Ok;
    } else {
      console.error(
        `[estimateOrderFees] Failed to calculate fees. Error: ${estimateOrderFees.Err}`,
      );
      throw new Error('Failed to calculate order fees');
    }
  } catch (error) {
    console.error('[estimateOrderFees] Error:', error);
    throw error;
  }
};

export const handleWeb3Error = (error: any): string => {
  console.log('Web3 error:', error);

  if (ethers.isError(error, 'CALL_EXCEPTION')) {
    return 'Transaction reverted. Please check if the token contract allows this operation or if you have enough balance.';
  } else if (ethers.isError(error, 'INSUFFICIENT_FUNDS')) {
    return 'Transaction failed due to insufficient funds. Please make sure your wallet has enough ETH for gas fees.';
  } else if (ethers.isError(error, 'NONCE_EXPIRED')) {
    return 'Transaction failed due to a nonce error. Try resubmitting the transaction.';
  } else if (ethers.isError(error, 'ACTION_REJECTED')) {
    return 'Transaction failed, user rejected the signature.';
  } else if (error.code) {
    return `Error code: ${error.code}. Please check the details and try again.`;
  } else {
    return `Transaction failed: ${
      error.message || 'Unknown error occurred. Please try again.'
    }`;
  }
};
