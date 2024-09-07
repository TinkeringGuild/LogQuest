import { Action } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/tauri';
import { clamp } from 'lodash';

import { Bootstrap } from './generated/Bootstrap';
import { DataDelta } from './generated/DataDelta';
import { LogQuestConfig } from './generated/LogQuestConfig';
import { Mutation } from './generated/Mutation';
import { OverlayState } from './generated/OverlayState';
import { TimerLifetime } from './generated/TimerLifetime';
import { TriggerIndex } from './generated/TriggerIndex';
import { UUID } from './generated/UUID';

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

export async function createTriggerTag(name: string): Promise<DataDelta[]> {
  return mutate([{ variant: 'CreateTriggerTag', value: name }]);
}

export async function addTriggerToTag(
  trigger_id: UUID,
  trigger_tag_id: UUID
): Promise<DataDelta[]> {
  return mutate([
    { variant: 'TagTrigger', value: { trigger_id, trigger_tag_id } },
  ]);
}

export async function removeTriggerFromTag(
  trigger_id: UUID,
  trigger_tag_id: UUID
): Promise<DataDelta[]> {
  return mutate([
    { variant: 'UntagTrigger', value: { trigger_id, trigger_tag_id } },
  ]);
}

export async function mutate(mutations: Mutation[]): Promise<DataDelta[]> {
  return await invoke('mutate', {
    mutations,
  });
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
): Promise<TriggerIndex> {
  return await invoke<TriggerIndex>('import_gina_triggers_file', {
    path: filePath,
  });
}
