import { getConfig } from "../config";
import { viewFunction } from "./utils";

export class Intents {
  private intentsContract: string;

  constructor() {
    const config = getConfig();
    this.intentsContract = config.near.contract.intents;
  }

  public async getBalances(accountId: string, tokenIds: string[]): Promise<string[]> {
    const result = await viewFunction({
      contractId: this.intentsContract,
      methodName: 'mt_batch_balance_of',
      args: {
        account_id: accountId,
        token_ids: tokenIds,
      },
    });
    const balances = result as string[];
    if (balances?.length !== tokenIds.length) {
      throw new Error(`Expected to receive ${tokenIds.length} balances, but got ${balances?.length}`);
    }
    return balances;
  }
}


export const INTENTS_TOKENS = {
  wNEAR: "nep141:wrap.near",
  USDC: "nep141:17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1",
  USDT: "nep141:usdt.tether-token.near",
}

export const TOKEN_INFO = {
  [INTENTS_TOKENS.wNEAR]: {
    defuseAssetId: "nep141:wrap.near",
    address: "wrap.near",
    decimals: 24,
    icon: "https://s2.coinmarketcap.com/static/img/coins/128x128/6535.png",
    chainName: "near",
    bridge: "direct",
    symbol: "NEAR",
    name: "Near",
  },
  [INTENTS_TOKENS.USDC]: {
    defuseAssetId:
      "nep141:17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1",
    address:
      "17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1",
    decimals: 6,
    icon: "https://s2.coinmarketcap.com/static/img/coins/128x128/3408.png",
    chainName: "near",
    bridge: "direct",
    symbol: "USDC",
    name: "USD Coin",
  },
  [INTENTS_TOKENS.USDT]: {
    defuseAssetId: "nep141:usdt.tether-token.near",
    address: "usdt.tether-token.near",
    decimals: 6,
    icon: "https://s2.coinmarketcap.com/static/img/coins/128x128/825.png",
    chainName: "near",
    bridge: "direct",
    symbol: "USDT",
    name: "Tether USD",
  },
}
