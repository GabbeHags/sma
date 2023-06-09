import { invoke } from '@tauri-apps/api/tauri';

export interface IRustConfig {
  version: number;
  cwd: string | null;
  cascadeKill: boolean;
  start: string[];
  exitOn: number | null;
}

export async function rustLoadConfigFile(configPath: string): Promise<IRustConfig> {
  return await invoke('load_config', { configPath });
}
