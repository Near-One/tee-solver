import { ENV } from './env';
import { Config } from './types';

export function getConfig(): Config {
  return require(`./${ENV}`).default;
}
