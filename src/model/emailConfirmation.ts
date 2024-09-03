import { v4 as uuidv4 } from 'uuid';

import {
  LoginAddress,
  PaymentProvider,
} from '../declarations/backend/backend.did';
import { UserTypes } from './types';

type TempUserData = {
  providers: Array<PaymentProvider>;
  userType: UserTypes;
  loginMethod: LoginAddress;
  password: string;
  confirmationToken: string;
};

type TempResetPasswordData = {
  loginMethod: LoginAddress;
  confirmationToken: string;
};

const storeTempUserData = (data: TempUserData) => {
  localStorage.setItem('tempUserData', JSON.stringify(data));
};

const storeTempResetPasswordData = (data: TempResetPasswordData) => {
  localStorage.setItem('tempResetPasswordData', JSON.stringify(data));
};

const sendConfirmationEmail = async (email: string, token: string) => {
  const requestBody = {
    to: email,
    token: token,
    domain: window.location.origin,
  };

  const secretToken = process.env.FRONTEND_EMAIL_TOKEN;
  if (!secretToken) {
    throw new Error(`Error: access token for email server is required`);
  }

  const response = await fetch(
    `${process.env.FRONTEND_EMAIL_SERVER}/send-confirmation-email`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-access-token': secretToken,
      },
      body: JSON.stringify(requestBody),
    },
  );

  if (!response.ok) {
    throw new Error(`Error: ${response.statusText}`);
  }

  await response.json();
};

const sendRecoverPassword = async (email: string, token: string) => {
  const secretToken = process.env.FRONTEND_EMAIL_TOKEN;
  if (!secretToken) {
    throw new Error(`Error: access token for email server is required`);
  }

  const requestBody = {
    to: email,
    resetToken: token,
    domain: window.location.origin,
  };

  try {
    const response = await fetch(
      `${process.env.FRONTEND_EMAIL_SERVER}/send-password-reset-email`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-access-token': secretToken,
        },
        body: JSON.stringify(requestBody),
      },
    );

    if (!response.ok) {
      throw new Error(`API request failed with status ${response.status}`);
    }

    return response;
  } catch (error) {
    console.error('Error in send password reset email:', error);
    throw error;
  }
};

const getTempUserData = (): TempUserData | null => {
  const data = localStorage.getItem('tempUserData');
  return data ? JSON.parse(data) : null;
};

const getTempResetPasswordData = (): TempResetPasswordData | null => {
  const data = localStorage.getItem('tempResetPasswordData');
  return data ? JSON.parse(data) : null;
};

const clearTempUserData = () => {
  localStorage.removeItem('tempUserData');
};

const clearTempResetPasswordData = () => {
  localStorage.removeItem('tempResetPasswordData');
};

const generateConfirmationToken = (): string => {
  return uuidv4();
};

export {
  storeTempUserData,
  storeTempResetPasswordData,
  getTempUserData,
  getTempResetPasswordData,
  clearTempUserData,
  clearTempResetPasswordData,
  generateConfirmationToken,
  sendConfirmationEmail,
  sendRecoverPassword,
};
