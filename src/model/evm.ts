import { ethers } from 'ethers';
import { icP2PrampABI } from '../constants/ic2P2ramp';
import { getVaultAddress } from '../constants/addresses';
import { MethodGasUsage, Order } from '../declarations/backend/backend.did';
import { TokenOption } from '../constants/tokens';
import { backend } from '../declarations/backend';

export const depositInVault = async (
  chainId: number,
  selectedToken: TokenOption,
  cryptoAmount: bigint,
) => {
  if (!window.ethereum) {
    throw new Error('No crypto wallet found. Please install it.');
  }

  const provider = new ethers.BrowserProvider(window.ethereum);
  await provider.send('eth_requestAccounts', []);
  const signer = await provider.getSigner();

  const vaultContract = new ethers.Contract(
    getVaultAddress(chainId),
    icP2PrampABI,
    signer,
  );

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
    throw new Error('Transaction failed!');
  }

  return receipt;
};

export const estimateGasAndGasPrice = async (
  chainId: number,
  method: MethodGasUsage,
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
  fiatAmount: bigint,
  cryptoAmount: bigint,
  token: [] | [string],
  gasForCommit: bigint,
  gasForRelease: bigint,
): Promise<[bigint, bigint]> => {
  try {
    const estimateOrderFees = await backend.calculate_order_evm_fees(
      chainId,
      fiatAmount,
      cryptoAmount,
      token,
      gasForCommit,
      gasForRelease,
    );

    if ('Ok' in estimateOrderFees) {
      return estimateOrderFees.Ok;
    } else {
      console.error('[estimateOrderFees] Failed to calculate fees');
      throw new Error('Failed to calculate order fees');
    }
  } catch (error) {
    console.error('[estimateOrderFees] Error:', error);
    throw error;
  }
};
