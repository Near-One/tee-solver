import { execSync } from 'child_process';
import { writeFileSync } from 'fs';
import { join } from 'path';
import { getConfig } from '../config';
import { getApiKey, saveApiKey } from './utils/credentials';
import { logger } from '../utils/logger';
import { CvmInstance } from './api/types';
import { getCvms } from './api/cvms';

const SOLVER_POOL_PREFIX = 'solver-pool-';
const SOLVER_PORT = 3000;

export class PhalaCloudService {
  async setupPhalaAuth(): Promise<void> {
    const config = getConfig();
    
    const localApiKey = await getApiKey();
    if (!localApiKey) {
        if (config.phala.apiKey) {
            await saveApiKey(config.phala.apiKey);
        } else {
            throw new Error('PHALA_CLOUD_API_KEY is not set');
        }
    }

    logger.info('Phala auth configured successfully');
  }

  async createSolverCvm(poolId: number, tokenIds: string[]): Promise<void> {
    const config = getConfig();

    // const composePath = join(process.cwd(), `docker-compose.yaml`);
    const envPath = join(process.cwd(), `.env.phala`);
    const envContent = `
NEAR_NETWORK_ID=${config.near.networkId}
INTENTS_CONTRACT=${config.near.contract.intents}
SOLVER_REGISTRY_CONTRACT=${config.near.contract.solverRegistry}
SOLVER_POOL_ID=${poolId}
AMM_TOKEN1_ID=${tokenIds[0]}
AMM_TOKEN2_ID=${tokenIds[1]}
    `;
    writeFileSync(envPath, envContent);

    // const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const cvmName = this.getSolverCvmName(poolId);
    const output = execSync(`npx phala cvms create -n ${cvmName} -c docker-compose.yaml -e .env.phala`, { encoding: 'utf-8' });

    logger.info(output);

    // const match = output.match(/CVM host URL: (https:\/\/[^\s]+)/);
    // if (!match) {
    //   throw new Error('Failed to extract CVM host URL from output');
    // }

    // return match[1];
  }

  getSolverCvmName(poolId: number): string {
    return `${SOLVER_POOL_PREFIX}${poolId}`;
  }

  async getSolverCvms(): Promise<CvmInstance[]> {
    const cvms = await getCvms();
    const solverCvms = cvms.filter(cvm => cvm.name && cvm.name.startsWith(SOLVER_POOL_PREFIX));
    return solverCvms;
  }

  async getSolverUrl(cvm: CvmInstance): Promise<string> {
    return `https://${cvm.hosted?.app_id}-${SOLVER_PORT}.dstack-${cvm.node.name}.phala.network`;
  }

  async getSolverAddress(solverUrl: string): Promise<string> {
    try {
      const response = await fetch(`${solverUrl}/address`);
      const data = await response.json() as { address: string };
      return data.address;
    } catch (error) {
      logger.error('Failed to get solver address:', error);
      throw error;
    }
  }
}
