export type Config = {
  near: {
    networkId: 'mainnet' | 'testnet';
    rpcUrl: string;
    contract: {
      intents: string;
      solverRegistry: string;
    };
    account: {
      operatorAddress: string;
      operatorPrivateKey: `ed25519:${string}`;
    };
  };
  phala: {
    apiKey: string;
  };
  worker: {
    minimumBalance: number;
  };
  pool: {
    minimumNearBalance: number;
    minimumStableCoinBalance: number;
  };
};
