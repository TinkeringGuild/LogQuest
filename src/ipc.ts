import { Action } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/tauri';
import { clamp } from 'lodash';
import { v4 as uuid } from 'uuid';

import { Bootstrap } from './generated/Bootstrap';
import { Character } from './generated/Character';
import { CommandTemplate } from './generated/CommandTemplate';
import { CommandTemplateSecurityCheck } from './generated/CommandTemplateSecurityCheck';
import { DataDelta } from './generated/DataDelta';
import { LogQuestConfig } from './generated/LogQuestConfig';
import { Mutation } from './generated/Mutation';
import { OverlayState } from './generated/OverlayState';
import { SystemCommandInfo } from './generated/SystemCommandInfo';
import { TimerLifetime } from './generated/TimerLifetime';
import { Trigger } from './generated/Trigger';
import { TriggerGroup } from './generated/TriggerGroup';
import { TriggerIndex } from './generated/TriggerIndex';
import { UUID } from './generated/UUID';
import { nowTimestamp } from './util';

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

export async function createTrigger(
  trigger: Trigger,
  trigger_tag_ids: UUID[],
  parent_position: number
): Promise<DataDelta[]> {
  return mutate([
    {
      variant: 'CreateTrigger',
      value: { trigger, trigger_tag_ids, parent_position },
    },
  ]);
}

export async function saveTrigger(
  trigger: Trigger,
  trigger_tag_ids: UUID[]
): Promise<DataDelta[]> {
  return mutate([
    { variant: 'SaveTrigger', value: { trigger, trigger_tag_ids } },
  ]);
}

export async function deleteTrigger(triggerID: UUID): Promise<DataDelta[]> {
  return mutate([{ variant: 'DeleteTrigger', value: triggerID }]);
}

export async function createTriggerGroup(
  name: string,
  comment: string | null,
  parent_id: UUID | null,
  parent_position: number
): Promise<DataDelta[]> {
  const now = nowTimestamp();

  const trigger_group: TriggerGroup = {
    id: uuid(),
    name,
    comment,
    parent_id,
    created_at: now,
    updated_at: now,
    children: [],
  };

  return mutate([
    {
      variant: 'CreateTriggerGroup',
      value: {
        trigger_group,
        parent_position,
      },
    },
  ]);
}

export async function saveTriggerGroup(
  trigger_group_id: UUID,
  name: string,
  comment: string | null
): Promise<DataDelta[]> {
  return mutate([
    { variant: 'SaveTriggerGroup', value: { trigger_group_id, name, comment } },
  ]);
}

export async function deleteTriggerGroup(groupID: UUID): Promise<DataDelta[]> {
  return mutate([{ variant: 'DeleteTriggerGroup', value: groupID }]);
}

export async function createTriggerTag(name: string): Promise<DataDelta[]> {
  return mutate([{ variant: 'CreateTriggerTag', value: name }]);
}

export async function renameTriggerTag(
  triggerTagId: UUID,
  name: string
): Promise<DataDelta[]> {
  return mutate([{ variant: 'RenameTriggerTag', value: [triggerTagId, name] }]);
}

export async function deleteTriggerTag(
  triggerTagID: UUID
): Promise<{ activeTriggerTags: UUID[]; deltas: DataDelta[] }> {
  const activeTriggerTags = await setTriggerTagActivated(triggerTagID, false);
  const deltas = await mutate([
    { variant: 'DeleteTriggerTag', value: triggerTagID },
  ]);
  return { activeTriggerTags, deltas };
}

export async function getActiveTriggerTags(): Promise<UUID[]> {
  return invoke<UUID[]>('get_active_trigger_tags');
}

export async function setTriggerTagActivated(
  id: UUID,
  activated: boolean
): Promise<UUID[]> {
  return invoke<UUID[]>('set_trigger_tag_activated', { id, activated });
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

export async function getSystemCommandInfo(
  command: string
): Promise<SystemCommandInfo> {
  return await invoke<SystemCommandInfo>('sys_command_info', { command });
}

export async function signCommandTemplate(
  cmdTmpl: CommandTemplate
): Promise<CommandTemplateSecurityCheck> {
  return await invoke<CommandTemplateSecurityCheck>('sign_command_template', {
    cmdTmpl,
  });
}

export type ValidateGINARegexResponse = [number | null, string] | null;

export async function validateGINARegex(
  pattern: string
): Promise<ValidateGINARegexResponse> {
  return await invoke<ValidateGINARegexResponse>('validate_gina_regex', {
    pattern,
  });
}

export async function validateGINARegexWithContext(
  pattern: string
): Promise<ValidateGINARegexResponse> {
  return await invoke<ValidateGINARegexResponse>(
    'validate_gina_regex_with_context',
    {
      pattern,
    }
  );
}

export async function playAudioFile(path: string) {
  await invoke('play_audio_file', { path });
}

export async function getCurrentCharacter() {
  return await invoke<Character | null>('get_current_character');
}
