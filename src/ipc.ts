import { Action } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/tauri';
import { clamp } from 'lodash';

import { Bootstrap } from './generated/Bootstrap';
import { LogQuestConfig } from './generated/LogQuestConfig';
import { OverlayState } from './generated/OverlayState';
import { TimerLifetime } from './generated/TimerLifetime';
import { TriggerRoot } from './generated/TriggerRoot';

export async function getBootstrap(): Promise<Bootstrap> {
  return await invoke<Bootstrap>('bootstrap');
}

export async function getOverlayBootstrap(): Promise<OverlayState> {
  return await invoke<OverlayState>('bootstrap_overlay');
}

export async function dispatchToOverlay(action: Action) {
  return await invoke('dispatch_to_overlay', { action });
}

export async function startTimersSync(): Promise<TimerLifetime[]> {
  return await invoke<TimerLifetime[]>('start_timers_sync');
}

export function invokeSetOverlayOpacity(newValue: number) {
  return invoke('set_overlay_opacity', {
    opacity: clamp(newValue, 0, 100),
  });
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
