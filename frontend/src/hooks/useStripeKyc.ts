import { ActorSubclass } from '@dfinity/agent';
import {
  _SERVICE,
  LoginAddress,
  PaymentProvider,
} from '@/declarations/icramp_backend/icramp_backend.did';
import { backend } from '@/model/backendProxy';
import { storeTempUserData } from '@/model/emailConfirmation';
import { rampErrorToString } from '@/model/helpers/error';
import { mapCountryToPlatform } from '@/utils/stripe';

type StartArgs = {
  userType: 'Offramper' | 'Onramper' | 'Visitor';
  loginMethod: LoginAddress | null;
  backendActor?: ActorSubclass<_SERVICE> | null;
  email: string; // providerId
  country: string; // 'ES', 'US', ...
  providers?: PaymentProvider[]; // only needed in Register page (to store temp)
  password?: string; // only needed in Register page
  storeUserData: boolean;
  setMessage: (s: string) => void;
  setLoading: (b: boolean) => void;
};

export async function startStripeKyc({
  userType,
  loginMethod,
  backendActor,
  email,
  country,
  providers,
  password,
  storeUserData,
  setMessage,
  setLoading,
}: StartArgs) {
  setMessage('');
  if (userType !== 'Offramper') {
    setMessage('Stripe is only used for Offrampers.');
    return;
  }
  if (!loginMethod) {
    setMessage('Please login first.');
    return;
  }
  if (!email || !country) {
    setMessage('Email and country are required for Stripe onboarding.');
    return;
  }

  const actor =
    'ICP' in (loginMethod ?? {}) && backendActor ? backendActor : backend;

  // optional (Register): preserve temp state for post-redirect
  if (storeUserData) {
    storeTempUserData({
      providers: providers ?? [],
      userType,
      loginMethod,
      password: password ?? '',
      confirmationToken: '',
    });
  }

  setLoading(true);
  const platform = mapCountryToPlatform(country);
  try {
    const r1 = await actor.stripe_create_express_account(email, country, [
      platform,
    ]);
    if ('Err' in r1) {
      setMessage(`Stripe create account failed: ${rampErrorToString(r1.Err)}`);
      return;
    }
    const acct = r1.Ok;

    const origin = window.location.origin;
    const returnUrl = `${origin}${
      window.location.pathname
    }?stripe_acct=${encodeURIComponent(acct)}&platform=${encodeURIComponent(
      platform,
    )}`;
    const refreshUrl = `${origin}${
      window.location.pathname
    }?refresh=1&stripe_acct=${encodeURIComponent(
      acct,
    )}&platform=${encodeURIComponent(platform)}`;

    const r2 = await actor.stripe_create_account_link(
      acct,
      refreshUrl,
      returnUrl,
      [platform],
    );
    if ('Err' in r2) {
      setMessage(`Stripe account link failed: ${rampErrorToString(r2.Err)}`);
      return;
    }

    window.location.href = r2.Ok;
  } catch (e: any) {
    setMessage(`Stripe onboarding failed: ${e?.message ?? String(e)}`);
  } finally {
    setLoading(false);
  }
}
