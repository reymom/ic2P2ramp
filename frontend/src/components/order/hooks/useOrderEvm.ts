import type {
  DepositInput,
  EvmOrderInput,
} from '@/declarations/icramp_backend/icramp_backend.did';
import type { TokenOption } from '@/model/types';
import {
  estimateGasAndGasPrice,
  depositInVault,
  sendEvmPayment,
} from '@/model/blockchain/evm';
import {
  defaultCommitEvmGas,
  defaultReleaseEvmGas,
} from '@/constants/evm_tokens';

export const useOrderEvm = () => {
  const makeEvmDeposit = async (
    chainId: number,
    token: TokenOption,
    amount: bigint,
  ) => {
    const gasForCommit = await estimateGasAndGasPrice(
      chainId,
      { Commit: null },
      defaultCommitEvmGas,
    );
    const txVariant = token.isNative ? { Native: null } : { Token: null };
    const gasForRelease = await estimateGasAndGasPrice(
      chainId,
      { Release: txVariant },
      defaultReleaseEvmGas,
    );

    const receipt = await depositInVault(chainId, token, amount);

    const depositInput: [DepositInput] = [
      {
        Evm: {
          estimated_gas_lock: gasForCommit[0],
          estimated_gas_withdraw: gasForRelease[0],
          tx_hash: receipt.hash,
        } as EvmOrderInput,
      },
    ];

    return { depositInput, txHash: receipt.hash };
  };

  const makeEvmCryptoPayment = async (
    chainId: number,
    token: TokenOption,
    amount: bigint,
    toAddress: string,
  ) => {
    const receipt = await sendEvmPayment(chainId, token, amount, toAddress);

    const depositInput: [DepositInput] = [
      {
        Evm: {
          // we don't care here; backend can ignore these
          estimated_gas_lock: 0n,
          estimated_gas_withdraw: 0n,
          tx_hash: receipt.hash,
        },
      },
    ];

    return { depositInput, txHash: receipt.hash };
  };

  return { makeEvmDeposit, makeEvmCryptoPayment };
};
