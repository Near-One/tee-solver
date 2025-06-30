import { getConfig } from '../config';
import { connect, KeyPair, keyStores, Near } from 'near-api-js';
import { parseNearAmount } from 'near-api-js/lib/utils/format';
import { ViewFunctionCallOptions } from 'near-api-js/lib/account';

export type NearService = {
  near: Near;
  keyStore: keyStores.InMemoryKeyStore;
};

export async function initNear(): Promise<NearService> {
  const config = getConfig();
  const keyStore = new keyStores.InMemoryKeyStore();
  // configure operator account's key
  keyStore.setKey(
    config.near.networkId,
    config.near.account.operatorAddress,
    KeyPair.fromString(config.near.account.operatorPrivateKey)
  );
  const near = await connect({
    networkId: config.near.networkId,
    nodeUrl: config.near.rpcUrl,
    keyStore,
  });
  return { near, keyStore };
}

export async function viewFunction(options: ViewFunctionCallOptions) {
  const { near } = await initNear();
  const account = await near.account("");
  return account.viewFunction(options);
}

export async function transfer(senderId: string, receiverId: string, amount: number) {
  const { near } = await initNear();
  const account = await near.account(senderId);
  const amountInYocto = parseNearAmount(amount.toString());
  if (!amountInYocto) {
    throw new Error("Invalid amount");
  }
  return account.sendMoney(receiverId, BigInt(amountInYocto));
}

export async function getBalance(accountId: string): Promise<string> {
  const { near } = await initNear();
  const account = await near.account(accountId);

  let balance = '0';
  try {
    const { available } = await account.getAccountBalance();
    balance = available;
  } catch (e: unknown) {
    if (e instanceof Error && 'type' in e && e.type === 'AccountDoesNotExist') {
      // this.logger.info(e.type);
    } else {
      throw e;
    }
  }
  return balance;
}
