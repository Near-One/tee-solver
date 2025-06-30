import { getConfig } from "../config";
import { range } from "../utils/array";
import { logger } from "../utils/logger";
import { Intents, INTENTS_TOKENS, TOKEN_INFO } from "./intents";
import { viewFunction } from "./utils";
import Big from "big.js";

const config = getConfig();

export interface WorkerInfo {
  account_id: string;
  pool_id: number;
  checksum: string;
  codehash: string;
}

export interface PoolInfo {
  /// List of tokens in the pool.
  token_ids: string[],
  /// How much NEAR this contract has.
  amounts: string[],
  /// Fee charged for swap in basis points
  fee: number,
  /// Total number of shares.
  shares_total_supply: string,
}

export class SolverRegistry {
  private solverRegistryContract: string | null = null;
  private intents: Intents | null = null;

  constructor() {
    if (this.solverRegistryContract) {
      return;
    }
    this.solverRegistryContract = config.near.contract.solverRegistry;
    this.intents = new Intents();
  }

  public async getPoolLen(): Promise<number> {
    return await viewFunction({
      contractId: this.solverRegistryContract!,
      methodName: 'get_pool_len',
      args: {},
    });
  }

  public async getPool(poolId: number): Promise<PoolInfo> {
    return await viewFunction({
      contractId: this.solverRegistryContract!,
      methodName: 'get_pool',
      args: { pool_id: poolId },
    });
  }

  public async getPoolContractId(poolId: number): Promise<string> {
    return `pool-${poolId}.${this.solverRegistryContract}`;
  }

  public async getPoolBalances(poolId: number): Promise<Record<string, string>> {
    const poolContractId = await this.getPoolContractId(poolId);
    const pool = await this.getPool(poolId);
    const tokenIds = pool.token_ids.map(tokenId => `nep141:${tokenId}`);
    const balances = await this.intents?.getBalances(poolContractId, tokenIds) ?? [];
    return tokenIds.reduce((acc, tokenId, index) => {
      acc[tokenId] = balances[index];
      return acc;
    }, {} as Record<string, string>);
  }

  public async hasEnoughBalancesInPool(poolId: number): Promise<boolean> {
    const poolBalances = await this.getPoolBalances(poolId);
    logger.info(`Pool ${poolId} token balances: ${JSON.stringify(poolBalances, null, 2)}`);

    function hasBalance(tokenId: string): boolean {
      const minBalance = tokenId === INTENTS_TOKENS.wNEAR ? config.pool.minimumNearBalance : config.pool.minimumStableCoinBalance;
      return !!poolBalances[tokenId] && Big(poolBalances[tokenId]).gte(Big(minBalance).mul(Big(10).pow(TOKEN_INFO[tokenId].decimals)));
    }

    // At least one token in the pair is wNEAR or USDC or USDT with enough balances
    return hasBalance(INTENTS_TOKENS.wNEAR)
      || hasBalance(INTENTS_TOKENS.USDC)
      || hasBalance(INTENTS_TOKENS.USDT);
  }

  public async getWorkerLen(): Promise<number> {
    return await viewFunction({
      contractId: this.solverRegistryContract!,
      methodName: 'get_worker_len',
      args: {},
    });
  }

  public async getWorker(accountId: string): Promise<WorkerInfo> {
    return await viewFunction({
      contractId: this.solverRegistryContract!,
      methodName: 'get_worker',
      args: {
        account_id: accountId
      },
    });
  }

  public async getWorkers(): Promise<WorkerInfo[]> {
    const workerLen = await this.getWorkerLen();
    const limit = 100;
    const workers = [];
    for (let i = 0; i < workerLen; i += limit) {
      const _workers = await viewFunction({
        contractId: this.solverRegistryContract!,
        methodName: 'get_workers',
        args: {
          offset: i,
          limit,
        },
      });
      workers.push(..._workers);
    }
    return workers;
  }

  public async getPoolsWithoutWorkers(): Promise<number[]> {
    const poolLen = await this.getPoolLen();

    const workers = await this.getWorkers();
    const poolIds = new Set(workers.map((worker) => worker.pool_id));

    return range(poolLen).filter((poolId) => !poolIds.has(poolId));
  }
}
