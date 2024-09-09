import { ethers } from 'ethers';
import { icP2PrampABI } from '../constants/ic2P2ramp';
import { getVaultAddress } from '../constants/addresses';
import { Order } from '../declarations/backend/backend.did';
import { TokenOption } from '../constants/tokens';

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

export const withdrawFromVault = async (chainId: number, order: Order) => {
  if (!window.ethereum) {
    throw new Error('No crypto wallet found. Please install it.');
  }
  if (!('EVM' in order.crypto.blockchain))
    throw new Error('Order is not for EVM');

  const provider = new ethers.BrowserProvider(window.ethereum);
  await provider.send('eth_requestAccounts', []);
  const signer = await provider.getSigner();

  const vaultContractAddress = getVaultAddress(chainId);
  const vaultContract = new ethers.Contract(
    vaultContractAddress,
    icP2PrampABI,
    signer,
  );

  const isNative =
    order.crypto.token.length === 0 ||
    order.crypto.token[0] === ethers.ZeroAddress;

  let transactionResponse;
  if (isNative) {
    const gasEstimate = await vaultContract.withdrawBaseCurrency.estimateGas(
      order.crypto.amount,
    );
    transactionResponse = await vaultContract.withdrawBaseCurrency(
      order.crypto.amount,
      { gasLimit: gasEstimate },
    );
  } else {
    const tokenAddress = order.crypto.token[0];
    const gasEstimate = await vaultContract.withdrawToken.estimateGas(
      tokenAddress,
      order.crypto.amount,
    );
    transactionResponse = await vaultContract.withdrawToken(
      tokenAddress,
      order.crypto.amount,
      { gasLimit: gasEstimate },
    );
  }

  const receipt = await transactionResponse.wait();
  if (receipt.status !== 1) {
    throw new Error('Transaction failed!');
  }

  return receipt;
};

// this cannot work here because it is onlyIcpEvmCanister
export const estimateOrderLockGas = async (
  chainId: number,
  selectedToken: TokenOption,
  cryptoAmount: bigint,
): Promise<bigint> => {
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

  let gasEstimateLock: bigint;
  if (selectedToken.isNative) {
    gasEstimateLock = await vaultContract.commitDeposit.estimateGas(
      await signer.getAddress(),
      ethers.ZeroAddress,
      cryptoAmount,
    );
  } else if (selectedToken.address !== '') {
    gasEstimateLock = await vaultContract.commitDeposit.estimateGas(
      await signer.getAddress(),
      selectedToken.address,
      cryptoAmount,
    );
  } else {
    throw new Error('Invalid token selection.');
  }

  return gasEstimateLock;
};

//this cannot work here because it is onlyIcpEvmCanister
export const estimateOrderReleaseGas = async (
  chainId: number,
  selectedToken: TokenOption,
  cryptoAmount: bigint,
): Promise<bigint> => {
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

  let gasEstimateRelease: bigint;
  if (selectedToken.isNative) {
    gasEstimateRelease = await vaultContract.releaseBaseCurrency.estimateGas(
      await signer.getAddress(),
      await signer.getAddress(),
      cryptoAmount,
    );
  } else if (selectedToken.address !== '') {
    gasEstimateRelease = await vaultContract.releaseFunds.estimateGas(
      await signer.getAddress(),
      await signer.getAddress(),
      selectedToken.address,
      cryptoAmount,
    );
  } else {
    throw new Error('Invalid token selection.');
  }

  return gasEstimateRelease;
};
