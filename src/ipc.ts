import { invoke } from '@tauri-apps/api/tauri';

import { Bootstrap } from './generated/Bootstrap';
import { LogQuestConfig } from './generated/LogQuestConfig';
import { TriggerRoot } from './generated/TriggerRoot';
import { TimerLifetime } from './generated/TimerLifetime';

export async function getBootstrap(): Promise<Bootstrap> {
  return await invoke<Bootstrap>('bootstrap');
}

export async function startSync(): Promise<TimerLifetime[]> {
  return await invoke<TimerLifetime[]>('start_sync');
}

export async function setEverQuestDirectory(
  newDir: string
): Promise<LogQuestConfig> {
  return await invoke<LogQuestConfig>('set_everquest_dir', { newDir });
}

export async function importGinaTriggersFile(
  filePath: string
): Promise<TriggerRoot> {
  return await invoke<TriggerRoot>('import_gina_triggers_file', {
    path: filePath,
  });
}
