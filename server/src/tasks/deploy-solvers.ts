import { getConfig } from "../config";
import { getBalance, transfer } from "../near/utils";
import { SolverRegistry } from "../near/solver-registry";
import { PhalaCloudService } from "../phala/cvms";
import { logger } from "../utils/logger";

export async function deploySolvers() {
  logger.info('---- Deploying Solvers ---');

  const solverRegistry = new SolverRegistry();
  const poolsWithoutWorkers = await solverRegistry.getPoolsWithoutWorkers();
  logger.info(`Found ${poolsWithoutWorkers.length} pools without workers: [${poolsWithoutWorkers.join(', ')}]`);

  const phala = new PhalaCloudService();
  const cvms = await phala.getSolverCvms();
  const cvmNames = cvms.map(cvm => cvm.name);

  const poolsWithoutCvms = poolsWithoutWorkers.filter(poolId => !cvmNames.includes(phala.getSolverCvmName(poolId)));
  logger.info(`Found ${poolsWithoutCvms.length} pools without CVMs: [${poolsWithoutCvms.join(', ')}]`);

  for (const poolId of poolsWithoutCvms) {
    const pool = await solverRegistry.getPool(poolId);
    logger.info(`Deploying solver for pool ${poolId}: ${JSON.stringify(pool, null, 2)}`);

    // If the pool doesn't have enough balances in NEAR Intents, skip deploying solver CVM
    if (!await solverRegistry.hasEnoughBalancesInPool(poolId)) {
      logger.info(`Pool ${poolId} does not have enough balances, skipping`);
      continue;
    }

    await phala.createSolverCvm(poolId, pool.token_ids);
    setTimeout(fundSolvers, 60 * 1000);
  }

  setTimeout(async () => {
    await deploySolvers();
  }, 60 * 1000);
}

export async function fundSolvers() {
  logger.info('---- Funding Solvers ---');

  const config = getConfig();
  const phala = new PhalaCloudService();
  const solverRegistry = new SolverRegistry();
  
  const cvms = await phala.getSolverCvms();
  const names = cvms.map(cvm => cvm.name);
  logger.info(`Found ${cvms.length} solver CVMs: [\n\t${names.join('\n\t')}\n]`);

  let failure = 0;
  for (const cvm of cvms) {
    try {
      const solverUrl = await phala.getSolverUrl(cvm);
      const solverAddress = await phala.getSolverAddress(solverUrl);
  
      // If the worker has been registered, no funds are needed
      const worker = await solverRegistry.getWorker(solverAddress);
      if (!worker) {
        const balance = await getBalance(solverAddress);
        logger.info(`Solver ${solverAddress} balance: ${balance}`);
        if (balance === '0') {
          const amount = config.worker.minimumBalance;
          logger.info(`Funding solver ${solverAddress} with ${amount} NEAR`);
          await transfer(config.near.account.operatorAddress, solverAddress, amount);
        }
      } else {
        logger.info(`Worker ${solverAddress} already exists: ${JSON.stringify(worker, null, 2)}`);
      }
    } catch (e) {
      logger.error(`Failed to fund CVM ${cvm.name}: ${e}`);
      failure++;
    }
  }

  if (failure > 0) {
    logger.error(`Failed to fund ${failure} CVMs`);
    setTimeout(async () => {
      await fundSolvers();
    }, 60 * 1000);
  }
}
