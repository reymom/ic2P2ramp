import { useEffect, useState, useMemo, useRef } from 'react';
import { useNavigate, useLocation, useSearchParams } from 'react-router-dom';
import { ethers } from 'ethers';

import {
  DepositInput,
  OrderState,
  PaymentProvider,
  PaymentProviderType,
} from '@/declarations/icramp_backend/icramp_backend.did';
import { useUser } from '@/components/user/UserContext';
import { NetworkIds, NetworkProps } from '@/constants/networks';
import { getEvmTokens } from '@/constants/evm_tokens';
import { ICP_TOKENS } from '@/constants/icp_tokens';
import { backend } from '@/model/backendProxy';
import {
  PaymentProviderTypes,
  providerTypes,
  TokenOption,
} from '@/model/types';
import { fetchBitcoinTokenOptions } from '@/model/blockchain/bitcoin';
import {
  blockchainAssetToBlockchainType,
  paymentProviderTypeToString,
  providerToProviderType,
} from '@/model/helpers/types';
import { rampErrorToString } from '@/model/helpers/error';
import { getExplorerUrls } from '@/utils/explorers';
import { formatCryptoUnits, formatPrice } from '@/utils/formatters';
import { fetchOrderPrice } from '@/utils/rates';

import icpLogo from '@/assets/blockchains/icp-logo.svg';
import bitcoinLogo from '@/assets/blockchains/bitcoin-logo.svg';
import { fetchSolanaTokenOptions } from '@/model/blockchain/solana';
import { useOrderEvm } from './useOrderEvm';
import { useOrderSolana } from './useOrderSolana';
import { useOrderBitcoin } from './useOrderBitcoin';
import { useOrderIcp } from './useOrderIcp';

const defaultLoadingMessage = 'Processing Transaction';
const PRICE_DIFFERENCE_THRESHOLD = 0.025;
const CACHE_EXPIRY_MS = 1800000; // 30 min, but backend is caching it to 10 minutes
const LOCK_TIME_SECONDS = 1800;

export const useOrderLogic = (
  order: OrderState,
  refetchOrders: () => Promise<void>,
) => {
  const [committedProvider, setCommittedProvider] =
    useState<[PaymentProviderType, PaymentProvider]>();
  const [tokens, setTokens] = useState<TokenOption[]>([]);
  const [loadingTokens, setLoadingTokens] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [loadingMessage, setLoadingMessage] = useState(defaultLoadingMessage);
  const [currentPrice, setCurrentPrice] = useState<bigint | null>(null);
  const [loadingPrice, setLoadingPrice] = useState<boolean>(false);
  const [txHash, setTxHash] = useState<string | null>(null);
  const [topUpAmount, setTopUpAmount] = useState<number>(0);
  const [topUpLoading, setTopUpLoading] = useState(false);
  const [topUpTxHash, setTopUpTxHash] = useState<string | null>(null);

  const [message, setMessage] = useState<string | null>(null);
  const [remainingTime, setRemainingTime] = useState<number | null>(null);
  const [isPayable, setIsPayable] = useState<boolean>(false);
  const [loadingPayable, setLoadingPayable] = useState<boolean>(true);

  const { makeEvmDeposit } = useOrderEvm();
  const { makeSolanaDeposit, waitForSolanaConfirmation } = useOrderSolana();
  const { makeBitcoinDeposit } = useOrderBitcoin();
  const { makeIcpDeposit } = useOrderIcp();

  const { user, sessionToken, icpAgent, fetchBalances, refetchUser } =
    useUser();
  const navigate = useNavigate();
  const location = useLocation();
  const [searchParams] = useSearchParams();

  const processedStripeReturnRef = useRef<string | null>(null);

  const [orderState, setOrderState] = useState(order);
  const [lockRefetched, setLockedRefetched] = useState(0);

  const orderId = useMemo(() => {
    return 'Created' in orderState
      ? orderState.Created.id
      : 'Locked' in orderState
      ? orderState.Locked.base.id
      : null;
  }, [orderState]);

  const baseOrder = useMemo(() => {
    return 'Created' in orderState
      ? orderState.Created
      : 'Locked' in orderState
      ? orderState.Locked.base
      : null;
  }, [orderState]);

  const orderBlockchainAsset = useMemo(() => {
    return 'Created' in orderState
      ? orderState.Created.crypto.asset
      : 'Locked' in orderState
      ? orderState.Locked.base.crypto.asset
      : 'Completed' in orderState
      ? orderState.Completed.asset
      : null;
  }, [orderState]);

  useEffect(() => {
    let alive = true;

    const load = async () => {
      if (!orderBlockchainAsset) return;
      setLoadingTokens(true);
      try {
        let list: TokenOption[] = [];
        if ('Bitcoin' in orderBlockchainAsset) {
          list = await fetchBitcoinTokenOptions();
        } else if ('Solana' in orderBlockchainAsset) {
          list = await fetchSolanaTokenOptions();
        }
        if (alive) setTokens(list);
      } catch (err) {
        console.error('Failed to fetch tokens:', err);
      } finally {
        if (alive) setLoadingTokens(false);
      }
    };

    load();
    return () => {
      alive = false;
    };
  }, [orderBlockchainAsset]);

  const committedMessage = `Locked Order #${orderId}, refetching data`;
  const releasedMessage = `Order Verified and Funds Released. Refetching data`;
  const cancelledMessage = `Cancelled Order #${orderId}, refetching data`;

  const getToken = (): TokenOption | null => {
    if (!baseOrder || !orderBlockchainAsset) return null;

    if ('Bitcoin' in orderBlockchainAsset) {
      const runeId = orderBlockchainAsset.Bitcoin.rune_id;
      if (runeId.length === 0) {
        return (
          tokens.find((token) => {
            return token.isNative;
          }) ?? null
        );
      } else {
        return (
          tokens.find((token) => {
            return token.runeMetadata && token.runeMetadata.id === runeId[0];
          }) ?? null
        );
      }
    } else if ('EVM' in orderBlockchainAsset) {
      const tokens = getEvmTokens(Number(orderBlockchainAsset.EVM.chain_id));
      const tokenAddress = orderBlockchainAsset.EVM.token_address?.[0] ?? '';
      return (
        tokens.find((token) => {
          return token.address === tokenAddress;
        }) ?? null
      );
    } else if ('ICP' in orderBlockchainAsset) {
      const tokenAddress = orderBlockchainAsset.ICP.ledger_principal.toString();
      return (
        ICP_TOKENS.find((token) => {
          return token.address === tokenAddress;
        }) ?? null
      );
    } else if ('Solana' in orderBlockchainAsset) {
      const mint = orderBlockchainAsset.Solana.spl_token?.[0] ?? '';
      return (
        tokens.find((t) => {
          return t.address === mint;
        }) ?? null
      );
    } else {
      return null;
    }
  };

  const token = useMemo(() => {
    return getToken();
  }, [baseOrder, orderBlockchainAsset, tokens]);

  const fetchOrder = async (orderId: bigint) => {
    try {
      const res = await backend.get_order(orderId);
      if ('Ok' in res) {
        console.log('[fetchOrder] res.Ok = ', res.Ok);
        setOrderState(res.Ok);
        return res.Ok;
      } else {
        console.error('Error fetching order: ', res.Err);
        await refetchOrders();
      }
    } catch (err) {
      console.error('Error fetching order: ', err);
    }
  };

  useEffect(() => {
    let alive = true;

    const getCurrentPrice = async () => {
      if ('Locked' in orderState) {
        if (alive)
          setCurrentPrice(
            orderState.Locked.price + orderState.Locked.offramper_fee,
          );
        return;
      }
      if (!('Created' in orderState)) return;

      if (orderId != null) {
        const cached = localStorage.getItem(`order_${orderId}_price`);
        if (cached) {
          const { price, timestamp } = JSON.parse(cached);
          if (Date.now() - timestamp < CACHE_EXPIRY_MS) {
            if (alive) setCurrentPrice(BigInt(price));
            return;
          }
        }
      }

      if (!baseOrder || !token) return;

      try {
        if (alive) setLoadingPrice(true);

        const priceData = await fetchOrderPrice(
          baseOrder.currency,
          baseOrder.crypto,
        );

        if (priceData) {
          const [price, offramperFee] = priceData;
          const total = price + offramperFee;
          if (orderId != null) {
            localStorage.setItem(
              `order_${orderId}_price`,
              JSON.stringify({
                price: Number(total),
                timestamp: Date.now(),
              }),
            );
          }
          setCurrentPrice(total);
        } else {
          setCurrentPrice(null);
        }
      } catch (e) {
        console.error('fetchOrderPrice failed:', e);
        if (alive) setCurrentPrice(null);
      } finally {
        if (alive) setLoadingPrice(false);
      }
    };

    getCurrentPrice();
  }, [orderId, orderState, baseOrder, token]);

  useEffect(() => {
    let intervalId: NodeJS.Timeout | null = null;

    if (orderId && baseOrder && baseOrder.processing) {
      setIsLoading(true);
      setLoadingMessage('Processing');

      intervalId = setInterval(async () => {
        const updatedOrder = await fetchOrder(BigInt(orderId));
        if (!updatedOrder) {
          clearInterval(intervalId!);
          await refetchOrders();
          setIsLoading(false);
          return;
        }

        if (
          ('Created' in updatedOrder && updatedOrder.Created.processing) ||
          ('Locked' in updatedOrder && updatedOrder.Locked.base.processing)
        ) {
          return;
        }

        clearInterval(intervalId!);
        setIsLoading(false);
        await refetchOrders();
      }, 5000);
    }

    return () => {
      if (intervalId) clearInterval(intervalId);
    };
  }, [orderId, baseOrder]);

  useEffect(() => {
    if ('Locked' in orderState) {
      const calculateRemainingTime = () => {
        const currentTime = Number(Date.now() * 1_000_000);
        const expiryTime =
          Number(orderState.Locked.locked_at) +
          LOCK_TIME_SECONDS * 1_000_000_000;
        const timeLeftSeconds = (expiryTime - currentTime) / 1_000_000_000;

        if (
          !orderState.Locked.payment_done &&
          timeLeftSeconds <= 0 &&
          lockRefetched < 3
        ) {
          setLockedRefetched((prev) => prev + 1);
          setTimeout(() => {
            fetchOrder(orderState.Locked.base.id);
            refetchUser();
          }, 2500);
        }
        setRemainingTime(timeLeftSeconds > 0 ? timeLeftSeconds : null);
      };

      calculateRemainingTime();

      const timer = setInterval(calculateRemainingTime, 1000);
      return () => clearInterval(timer);
    }
  }, [orderState]);

  useEffect(() => {
    if (!('Locked' in orderState) || !sessionToken || !user) return;

    const orderId = orderState.Locked.base.id;
    const paymentId =
      orderState.Locked?.payment_id?.[0] ??
      orderState.Locked?.payment_id ??
      'stripe-return';

    const flagStripe = (searchParams.get('stripe') || '').toLowerCase();
    const flagSuccess = (searchParams.get('success') || '').toLowerCase();
    const redirectStatus = (
      searchParams.get('redirect_status') || ''
    ).toLowerCase();
    const spOrder = searchParams.get('order_id');

    const successHit =
      flagStripe === 'success' ||
      flagSuccess === 'true' ||
      redirectStatus === 'succeeded';

    const isThisOrder =
      !!spOrder && !!orderId && BigInt(spOrder) === BigInt(orderId);

    if (!(successHit && isThisOrder)) return;

    const seenKey = `stripe_verified_${spOrder}`;
    if (sessionStorage.getItem(seenKey) === '1') return;
    sessionStorage.setItem(seenKey, '1');

    if (processedStripeReturnRef.current === String(paymentId)) return;
    processedStripeReturnRef.current = String(paymentId);

    (async () => {
      try {
        setIsLoading(true);
        setLoadingMessage('Payment received. Verifying');
        const resp = await backend.verify_transaction(
          orderId,
          [sessionToken],
          'stripe',
        );
        if ('Ok' in resp) {
          if ('EVM' in orderBlockchainAsset!) {
            setTxHash(resp.Ok);
            await pollTransactionLog(orderId, user.id);
          } else if ('Bitcoin' in orderBlockchainAsset!) {
            setTxHash(resp.Ok);
            setLoadingMessage(
              'Bitcoin transaction is being processed. This may take some time to confirm (15-60+ minutes).',
            );
          } else {
            setLoadingMessage('Funds released');
            setTimeout(() => {
              fetchOrder(orderId);
              refetchUser();
              setIsLoading(false);
              fetchBalances();
              navigate('/view?status=Completed');
            }, 2500);
          }
        } else {
          setIsLoading(false);
          setMessage(rampErrorToString(resp.Err));
        }
      } catch (e) {
        setIsLoading(false);
        setMessage('Error verifying Stripe payment.');
        console.error(e);
      } finally {
        navigate(location.pathname, { replace: true });
      }
    })();
  }, [orderState, sessionToken, searchParams, orderId]);

  const handleProviderSelection = (
    selectedProviderType: PaymentProviderTypes,
  ) => {
    if (!user) return;

    const onramperProvider = user.payment_providers.find((userProvider) => {
      const p = paymentProviderTypeToString(
        providerToProviderType(userProvider),
      );
      return (
        p === selectedProviderType ||
        (p === 'Email' && selectedProviderType === 'Stripe')
      );
    });
    if (!onramperProvider) return;

    if (
      committedProvider &&
      paymentProviderTypeToString(committedProvider[0]) === selectedProviderType
    ) {
      setCommittedProvider(undefined);
    } else {
      const provider: [PaymentProviderType, PaymentProvider] = [
        providerToProviderType(onramperProvider),
        onramperProvider,
      ];
      setCommittedProvider(provider);
    }
  };

  const pollTransactionLog = async (
    orderId: bigint,
    userId: bigint,
    maxAttempts = 35,
  ) => {
    if (!sessionToken)
      throw new Error('Please authenticate to get a token session');

    let attempts = 0;
    let pollingTimer: NodeJS.Timeout | null = null;

    const clearPolling = () => {
      if (pollingTimer) clearTimeout(pollingTimer);
    };

    const pollLog = async () => {
      console.log(
        `[pollTransactionLog] Polling attempt: ${attempts}, maxAttempts: ${maxAttempts}`,
      );
      if (attempts >= maxAttempts) {
        clearPolling();
        setMessage(
          'Network seems to be very busy. Please check later or contact with the maintainer.',
        );
        fetchOrder(orderId);
        setIsLoading(false);
        return;
      }

      attempts += 1;

      try {
        const logResult = await backend.get_order_tx_log(orderId, [
          [userId, sessionToken],
        ]);
        console.log('[pollTransactionLog] logResult = ', logResult);

        if ('Ok' in logResult && logResult.Ok.length > 0 && logResult.Ok[0]) {
          const transactionLog = logResult.Ok[0];
          console.log('[pollTransactionLog] Transaction Log:', transactionLog);

          if ('Confirmed' in transactionLog.status) {
            const receipt = transactionLog.status.Confirmed;
            const successMessage =
              'Commit' in transactionLog.action
                ? committedMessage
                : 'Release' in transactionLog.action
                ? releasedMessage
                : 'Cancel' in transactionLog.action
                ? cancelledMessage
                : 'Transaction is successful!';

            setLoadingMessage(successMessage);
            setTxHash(receipt.transactionHash);
            setTimeout(() => {
              fetchOrder(orderId);
              refetchUser();
              fetchBalances();
              setIsLoading(false);
              navigate(
                'Commit' in transactionLog.action
                  ? `/view?onramperId=${user!.id}`
                  : 'Release' in transactionLog.action
                  ? '/view?status=Completed'
                  : 'Cancel' in transactionLog.action
                  ? '/view?status=Cancelled'
                  : '',
              );
            }, 3500);
            return;
          } else if ('Failed' in transactionLog.status) {
            console.log(
              '[pollTransactionLog] Transaction Failed:',
              transactionLog.status.Failed,
            );
            setMessage('Transaction failed. Please contact to maintainer.');
            setIsLoading(false);
            fetchOrder(orderId);
            clearPolling();
            return;
          } else if ('BroadcastError' in transactionLog.status) {
            console.log(
              '[pollTransactionLog] Broadcasting Error: ',
              transactionLog.status.BroadcastError,
            );
            setMessage(
              `Could not broadcast transaction. Please contact to maintainer.`,
            );
            setIsLoading(false);
            fetchOrder(orderId);
            clearPolling();
            return;
          } else if ('Unresolved' in transactionLog.status) {
            console.log(
              '[pollTransactionLog] Unresolved transaction: ',
              transactionLog.status.Unresolved,
            );
            setMessage(`Unresolved transaction. Please contact to maintainer.`);
            setIsLoading(false);
            fetchOrder(orderId);
            clearPolling();
            return;
          }
        } else if ('Err' in logResult) {
          setMessage(`Transaction failed. Please contact to maintainer.`);
          setIsLoading(false);
          setTxHash(null);
          fetchOrder(orderId);
          clearPolling();
          return;
        }

        // If still pending, poll again after a short delay
        pollingTimer = setTimeout(pollLog, 4000);
      } catch (error) {
        console.error('Error polling transaction logs: ', error);
        setMessage(
          'Failed to retrieve transaction status. Please contact to maintainer.',
        );
        fetchOrder(orderId);
        setIsLoading(false);
        clearPolling();
      }
    };

    pollLog();
    return () => clearPolling();
  };

  const checkIfOrderIsPayable = async (
    orderId: bigint,
    tokenSession: string,
  ): Promise<boolean> => {
    if (user && orderState && 'Locked' in orderState) {
      try {
        const result = await backend.verify_order_is_payable(
          orderId,
          tokenSession,
        );
        if ('Ok' in result) {
          if (orderState.Locked.onramper.user_id === user.id) {
            return true;
          }
          return false;
        } else {
          console.error('Order is not payable:', result.Err);
          return false;
        }
      } catch (error) {
        console.error('Error checking order payable status:', error);
        return false;
      }
    } else {
      return false;
    }
  };

  useEffect(() => {
    const validateOrderPayable = async () => {
      if (orderState && 'Locked' in orderState) {
        const payable = await checkIfOrderIsPayable(
          orderState.Locked.base.id,
          sessionToken!,
        );
        setIsPayable(payable);
        setLoadingPayable(false);
      }
    };

    if (orderState && sessionToken) {
      validateOrderPayable();
    } else {
      setLoadingPayable(false);
    }
  }, [sessionToken]);

  const commitToOrder = async (provider: PaymentProvider) => {
    if (!sessionToken)
      throw new Error('Please authenticate to get a token session');
    if (
      !user ||
      !('Onramper' in user.user_type) ||
      !('Created' in orderState) ||
      !orderBlockchainAsset ||
      !orderId
    )
      return;

    setIsLoading(true);
    setTxHash(null);
    setMessage(null);
    setLoadingMessage('Fetching order price');

    const priceData = await fetchOrderPrice(
      baseOrder!.currency,
      baseOrder!.crypto,
    );
    if (!priceData) {
      setMessage('Could not set order price');
      setIsLoading(false);
      return;
    }

    const [orderPrice, offramperFee] = priceData;
    const currentPriceNumber = Number(currentPrice);
    const totalOrderPrice = Number(orderPrice) + Number(offramperFee);
    const priceDifference = Math.abs(
      (totalOrderPrice - currentPriceNumber) / currentPriceNumber,
    );
    if (priceDifference > PRICE_DIFFERENCE_THRESHOLD) {
      const confirm = window.confirm(
        `The real price differs significantly from the previously estimated price. 
                    Real price: $${formatPrice(
                      totalOrderPrice,
                    )}, estimated price: $${formatPrice(currentPriceNumber)}. 
                    Do you want to proceed?`,
      );
      if (!confirm) {
        setIsLoading(false);
        return;
      }
    }

    const onramperAddress = user.addresses.find(
      (addr) => Object.keys(orderBlockchainAsset)[0] in addr.address_type,
    );
    if (!onramperAddress) {
      setIsLoading(false);
      setMessage('No address matches for user');
      return;
    }

    const onIsEmail = 'Email' in provider;
    const base = `${window.location.origin}${window.location.pathname}`;
    const successUrl = `${base}?stripe=success&order_id=${orderId}`;
    const cancelUrl = `${base}?stripe=cancel&order_id=${orderId}`;

    if (provider) setLoadingMessage('Locking Order');
    try {
      const result = await backend.lock_order(
        orderId,
        sessionToken,
        user.id,
        provider,
        onramperAddress,
        onIsEmail ? [successUrl] : [],
        onIsEmail ? [cancelUrl] : [],
      );
      if ('Ok' in result) {
        if ('EVM' in orderBlockchainAsset) {
          pollTransactionLog(orderId, user.id);
        } else {
          setLoadingMessage(committedMessage);
          setTimeout(() => {
            setIsLoading(false);
            fetchOrder(orderId);
            refetchUser();
            fetchBalances();
            navigate(`/view?onramperId=${user.id}`);
          }, 2500);
        }
      } else {
        setMessage(rampErrorToString(result.Err));
        setIsLoading(false);
      }
    } catch (err) {
      setMessage(`Error while committing to order ${orderId}.`);
      setIsLoading(false);
      console.error(err);
    }
  };

  const removeOrder = async () => {
    if (!sessionToken)
      throw new Error('Please authenticate to get a token session');
    if (
      !user ||
      !('Offramper' in user?.user_type) ||
      !('Created' in orderState) ||
      !orderBlockchainAsset ||
      !orderId
    )
      return;
    if (!baseOrder || user.id !== baseOrder.offramper_user_id) return;

    const scrollPosition = window.scrollY;

    setIsLoading(true);
    setTxHash(null);
    setMessage(null);
    setLoadingMessage(`Removing order ${orderId}`);

    try {
      const result = await backend.cancel_order(orderId, sessionToken);
      if ('Ok' in result) {
        if ('EVM' in orderBlockchainAsset) {
          setTxHash(result.Ok);
          pollTransactionLog(orderId, user.id);
        } else {
          setLoadingMessage(cancelledMessage);
          setTimeout(() => {
            fetchOrder(orderId);
            refetchUser();
            fetchBalances();
            setIsLoading(false);

            window.scrollTo(0, scrollPosition);
          }, 2500);
        }
      } else {
        setMessage(rampErrorToString(result.Err));
        setIsLoading(false);
      }
    } catch (err) {
      setMessage(`Error while removing order ${orderId}.`);
      setIsLoading(false);
      console.error(err);
    }
  };

  const toUnits = (n: number, dec: number) =>
    BigInt(Math.floor(n * Math.pow(10, dec)));

  const topUp = async () => {
    if (!baseOrder || !token || !user || !sessionToken || !orderBlockchainAsset)
      return;
    if (topUpAmount <= 0) return;

    setMessage(null);
    setTopUpTxHash(null);
    setTopUpLoading(true);

    try {
      const amountUnits = toUnits(topUpAmount, token.decimals);
      let depositInput: [] | [DepositInput] = [];

      // deposit on-chain / ledger first (same pattern as CreateOrder)
      if ('EVM' in orderBlockchainAsset) {
        const { depositInput: evmDep, txHash } = await makeEvmDeposit(
          Number(orderBlockchainAsset.EVM.chain_id),
          token,
          amountUnits,
        );
        setTopUpTxHash(txHash);
        depositInput = evmDep;
      } else if ('ICP' in orderBlockchainAsset) {
        if (!icpAgent) {
          throw new Error('ICP Agent not initialized');
        }
        await makeIcpDeposit(icpAgent, token, amountUnits);
      } else if ('Bitcoin' in orderBlockchainAsset) {
        const { depositInput: btcDep, txid } = await makeBitcoinDeposit(
          amountUnits,
          token,
        );
        setTopUpTxHash(txid);
        depositInput = btcDep;
      } else if ('Solana' in orderBlockchainAsset) {
        const { depositInput: solDep, txSig } = await makeSolanaDeposit(
          amountUnits,
          token,
        );
        setTopUpTxHash(txSig);
        await waitForSolanaConfirmation(txSig, { timeoutMs: 90_000 });
        depositInput = solDep;
      }

      // backend: top up (candid Option => [] | [value])
      const r = await backend.top_up_order(
        BigInt(baseOrder.id),
        BigInt(user.id),
        sessionToken,
        amountUnits,
        depositInput,
      );

      if ('Err' in r) throw new Error(rampErrorToString(r.Err));

      setTopUpAmount(0);
      await refetchOrders();
    } catch (e: any) {
      setMessage(e?.message ?? String(e));
    } finally {
      setTopUpLoading(false);
    }
  };

  const handlePayPalSuccess = async (transactionId: string) => {
    if (!sessionToken)
      throw new Error('Please authenticate to get a token session');
    if (!('Locked' in orderState) || !orderId || !orderBlockchainAsset) return;
    if (!user || !('Onramper' in user.user_type)) return;

    console.log('[handlePayPalSuccess] transactionID = ', transactionId);

    setIsLoading(true);
    setTxHash(null);
    setMessage(null);
    setLoadingMessage(`Payment received. Verifying`);

    try {
      // Send transaction ID to backend to verify payment
      const response = await backend.verify_transaction(
        orderId,
        [sessionToken],
        transactionId,
      );
      if ('Ok' in response) {
        if ('EVM' in orderBlockchainAsset!) {
          setTxHash(response.Ok);
          pollTransactionLog(orderId, user!.id);
        } else if ('Bitcoin' in orderBlockchainAsset!) {
          setTxHash(response.Ok);
          setLoadingMessage(
            'Bitcoin transaction is being processed. This may take some time to confirm (15-60+ minutes). You can check the status using the link below.',
          );
        } else {
          setLoadingMessage(releasedMessage);
          setTimeout(() => {
            fetchOrder(orderId);
            refetchUser();
            setIsLoading(false);
            fetchBalances();
            navigate('/view?status=Completed');
          }, 2500);
        }
      } else {
        setIsLoading(false);
        const errorMessage = rampErrorToString(response.Err);
        setMessage(errorMessage);
      }
    } catch (err) {
      setIsLoading(false);
      setMessage(`Error verifying payment for order ${orderId}.`);
      console.error(err);
    }
  };

  const handleRevolutRedirect = async () => {
    if (!sessionToken)
      throw new Error('Please authenticate to get a token session');
    if (!('Locked' in orderState) || !orderId) return;

    const consentUrl = orderState.Locked.revolut_consent[0]?.url;
    if (consentUrl) {
      console.log('Listening for Revolut transaction confirmation...');
      backend
        .execute_revolut_payment(orderId, sessionToken)
        .catch((err) =>
          console.error('Failed to execute revolut payment: ', err),
        );
      window.location.href = consentUrl;
    } else {
      console.error('Consent URL is not available.');
    }
  };

  const handleStripePay = () => {
    if (!('Locked' in orderState)) return;
    const payUrl =
      orderState.Locked?.payment_url?.[0] ?? orderState.Locked?.payment_url;

    if (!payUrl) {
      setMessage('Stripe payment link not available');
      return;
    }

    setIsLoading(true);
    setLoadingMessage('Redirecting to Stripe Checkout…');
    window.location.href = payUrl as string; // same-tab redirect
  };

  const getNetwork = (): NetworkProps | undefined => {
    return orderBlockchainAsset && 'EVM' in orderBlockchainAsset
      ? Object.values(NetworkIds).find(
          (network) => network.id === Number(orderBlockchainAsset.EVM.chain_id),
        )
      : undefined;
  };

  const getExplorerLinks = (orderAddress: string, txHash?: string) => {
    if (!orderBlockchainAsset) return null;

    const blockchainType = Object.keys(orderBlockchainAsset)[0];

    return getExplorerUrls(
      blockchainType,
      orderAddress,
      'EVM' in orderBlockchainAsset
        ? orderBlockchainAsset.EVM.chain_id
        : undefined,
      txHash ?? undefined,
    );
  };

  const getNetworkLogo = (): string | undefined => {
    if (!orderBlockchainAsset) return '';
    if ('EVM' in orderBlockchainAsset) {
      return getNetwork()!.logo;
    } else if ('ICP' in orderBlockchainAsset) {
      return icpLogo;
    } else if ('Bitcoin' in orderBlockchainAsset) {
      return bitcoinLogo;
    }
  };

  const getNetworkName = (): string | undefined => {
    if (!orderBlockchainAsset) return '';
    if ('EVM' in orderBlockchainAsset) {
      return getNetwork()!.name;
    } else if ('ICP' in orderBlockchainAsset) {
      return 'ICP';
    } else if ('Bitcoin' in orderBlockchainAsset) {
      return 'Bitcoin';
    }
  };

  const formatCryptoAmount = () => {
    if (!baseOrder || !orderBlockchainAsset || !token) return;
    let crypto = baseOrder.crypto;

    switch (blockchainAssetToBlockchainType(orderBlockchainAsset)) {
      case 'EVM':
        let fullAmountEVM: string;
        if (token.isNative) {
          fullAmountEVM = ethers.formatEther(crypto.amount - crypto.fee);
        } else {
          fullAmountEVM = ethers.formatUnits(
            (crypto.amount - crypto.fee).toString(),
            token.decimals,
          );
        }
        const shortAmountEVM = formatCryptoUnits(parseFloat(fullAmountEVM));
        return { fullAmount: fullAmountEVM, shortAmount: shortAmountEVM };
      case 'ICP':
      case 'Bitcoin':
      case 'Solana':
        const fullAmount = (
          Number(crypto.amount - crypto.fee) /
          10 ** token.decimals
        ).toString();
        const shortAmount = formatCryptoUnits(parseFloat(fullAmount));
        return { fullAmount, shortAmount };
      default:
        return {
          fullAmount: 'not implemented',
          shortAmount: 'not implemented',
        };
    }
  };

  const cryptoAmount = formatCryptoAmount();

  const getStatusColors = () => {
    let backgroundColor =
      'Created' in orderState
        ? 'bg-blue-200 bg-opacity-50 dark:bg-blue-900 dark:opacity-80'
        : 'Locked' in orderState
        ? 'bg-yellow-200 bg-opacity-50 dark:bg-yellow-900 dark:opacity-80'
        : 'Completed' in orderState
        ? 'bg-green-200 bg-opacity-50 dark:bg-green-900 dark:opacity-80'
        : 'Cancelled' in orderState
        ? 'bg-red-600 dark:opacity-50 dark:bg-red-900 dark:opacity-80'
        : 'bg-gray-200 bg-opacity-50 dark:bg-gray-900 dark:opacity-80';

    let borderColor =
      'Created' in orderState
        ? 'border-blue-300 dark:border-blue-600'
        : 'Locked' in orderState
        ? 'border-yellow-300 dark:border-yellow-600'
        : 'Completed' in orderState
        ? 'border-green-300 dark:border-green-600'
        : 'Cancelled' in orderState
        ? 'border-red-300 dark:border-red-600'
        : 'border-gray-300 dark:border-gray-600';

    let textColor =
      'Created' in orderState || 'Locked' in orderState
        ? 'text-gray-900 dark:text-white'
        : 'text-gray-700 dark:text-gray-300';

    return { backgroundColor, borderColor, textColor };
  };

  const getStatus = () => {
    return 'Created' in orderState
      ? 'Created'
      : 'Locked' in orderState
      ? 'Locked'
      : 'Completed' in orderState
      ? 'Completed'
      : 'Cancelled';
  };

  return {
    orderState,
    baseOrder,
    orderId,
    orderBlockchainAsset,
    token,
    cryptoAmount,
    currentPrice,
    isLoading,
    loadingMessage,
    message,
    txHash,
    loadingTokens,
    loadingPayable,
    loadingPrice,
    remainingTime,
    isPayable,
    committedProvider,
    topUpAmount,
    topUpLoading,
    topUpTxHash,
    getStatusColors,
    getStatus,
    getNetwork,
    getNetworkLogo,
    getNetworkName,
    getExplorerLinks,
    handleProviderSelection,
    commitToOrder,
    removeOrder,
    setTopUpAmount,
    topUp,
    handlePayPalSuccess,
    handleRevolutRedirect,
    handleStripePay,
    fetchOrder,
  };
};
