import { optionalEnv, requiredEnv } from "./env";
import { Config } from './types';

const config: Config = {
  near: {
    networkId: 'mainnet',
    rpcUrl: optionalEnv('NEAR_RPC_URL') || 'https://near.lava.build',
    contract: {
      intents: 'intents.near',
      solverRegistry: 'solver-registry-dev.near',
    },
    account: {
      operatorAddress: requiredEnv('FUNDING_ACCOUNT_ID'),
      operatorPrivateKey: requiredEnv('FUNDING_ACCOUNT_PRIVATE_KEY') as `ed25519:${string}`,
    },
  },
  phala: {
    apiKey: optionalEnv('PHALA_CLOUD_API_KEY') || '',
  },
  worker: {
    minimumBalance: 0.1, // NEAR
  },
  pool: {
    minimumNearBalance: 5, // wNEAR
    minimumStableCoinBalance: 10, // USDC
  },
};

export default config;
