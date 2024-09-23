import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { remove, some, sortBy } from 'lodash';
import { v4 as uuid } from 'uuid';

import { CommandTemplateSecurityCheck } from '../../generated/CommandTemplateSecurityCheck';
import { Duration } from '../../generated/Duration';
import { Effect } from '../../generated/Effect';
import { EffectWithID } from '../../generated/EffectWithID';
import { Filter } from '../../generated/Filter';
import { FilterWithContext } from '../../generated/FilterWithContext';
import { Matcher } from '../../generated/Matcher';
import { MatcherWithContext } from '../../generated/MatcherWithContext';
import { Timer } from '../../generated/Timer';
import { TimerEffect } from '../../generated/TimerEffect';
import { Trigger } from '../../generated/Trigger';
import { TriggerGroup } from '../../generated/TriggerGroup';
import { TriggerTag } from '../../generated/TriggerTag';
import { UUID } from '../../generated/UUID';
import {
  EffectVariant,
  isTimerEffectVariant,
  TimerEffectVariant,
} from '../../main/editor/effect-utils';
import { MainRootState } from '../../MainStore';
import { nowTimestamp } from '../../util';

export const TRIGGER_EDITOR_SLICE = 'trigger-editor';

export type TriggerEditorState = {
  draft: Trigger | null;
  draftTriggerTags: TriggerTag[];
  draftAncestors: TriggerGroup[];

  /// The presence of draftParentPosition indicates the Trigger being
  /// edited is a new Trigger. (When saving rather than creating, we
  /// don't need to send the parent position because it's not modified
  /// at save-time.)
  draftParentPosition: number | null;

  errors: { [key: string]: string };
};

const INITIAL_TRIGGER_EDITOR_STATE = {
  draft: null,
  draftTriggerTags: [],
  draftParentPosition: null,
  draftAncestors: [],
  errors: {},
} satisfies TriggerEditorState;

export type TriggerEditorSelector<T> = (slice: TriggerEditorState) => T;

type TimerField = keyof Timer;
type TimerFieldValue<T extends TimerField> = Timer[T];
interface SetTimerFieldPayload<T extends TimerField> {
  field: T;
  value: TimerFieldValue<T>;
  selector: TriggerEditorSelector<Timer>;
}

export type TimerEffectWaitUntilFilterMatchesType = Extract<
  TimerEffect,
  { variant: 'WaitUntilFilterMatches' }
>;

export type TimerEffectWaitUntilSecondsRemainType = Extract<
  TimerEffect,
  { variant: 'WaitUntilSecondsRemain' }
>;

export type EffectVariantCopyToClipboard = Extract<
  Effect,
  { variant: 'CopyToClipboard' }
>;

export type EffectVariantSpeak = Extract<Effect, { variant: 'Speak' }>;

export type EffectVariantOverlayMessage = Extract<
  Effect,
  { variant: 'OverlayMessage' }
>;

export type EffectVariantPlayAudioFile = Extract<
  Effect,
  { variant: 'PlayAudioFile' }
>;

export type EffectVariantRunSystemCommand = Extract<
  Effect,
  { variant: 'RunSystemCommand' }
>;

export type EffectVariantPause = Extract<Effect, { variant: 'Pause' }>;

const triggerEditorSlice = createSlice({
  name: TRIGGER_EDITOR_SLICE,
  initialState: INITIAL_TRIGGER_EDITOR_STATE,

  reducers: {
    editNewTrigger(
      slice: TriggerEditorState,
      {
        payload: { parentID, parentPosition, ancestorGroups },
      }: PayloadAction<{
        parentID: UUID | null;
        parentPosition: number;
        ancestorGroups: TriggerGroup[];
      }>
    ) {
      const now = nowTimestamp();
      slice.draft = {
        id: uuid(),
        parent_id: parentID,
        name: '',
        comment: null,
        effects: [],
        created_at: now,
        updated_at: now,
        filter: [],
        enabled: false, // TODO: REMOVE THIS FROM RUST CODE
      };
      slice.draftParentPosition = parentPosition;
      slice.draftTriggerTags = [];
      slice.draftAncestors = ancestorGroups;
    },

    editTriggerDraft(
      slice: TriggerEditorState,
      {
        payload: { trigger, triggerTags },
      }: PayloadAction<{ trigger: Trigger; triggerTags: TriggerTag[] }>
    ) {
      slice.draft = trigger;
      slice.draftTriggerTags = sortBy(triggerTags, (tag) =>
        tag.name.toUpperCase()
      );
    },

    cancelEditing(slice: TriggerEditorState) {
      Object.assign(slice, INITIAL_TRIGGER_EDITOR_STATE);
    },

    setTriggerName(
      slice: TriggerEditorState,
      { payload: name }: PayloadAction<string>
    ) {
      slice.draft!.name = name;
    },

    setTriggerComment(
      slice: TriggerEditorState,
      { payload: comment }: PayloadAction<string>
    ) {
      slice.draft!.comment = comment;
    },

    insertNewEffect(
      slice: TriggerEditorState,
      {
        payload: { variant, index, triggerID, seqSelector },
      }: PayloadAction<{
        variant: EffectVariant;
        index: number;
        triggerID: UUID;
        seqSelector: TriggerEditorSelector<EffectWithID[]>;
      }>
    ) {
      const seq = seqSelector(slice);
      let effect: EffectWithID = {
        id: uuid(),
        effect: newEffect(variant, triggerID),
      };
      seq.splice(index, 0, effect);
    },

    deleteEffect(
      slice: TriggerEditorState,
      {
        payload: { effectID, selector },
      }: PayloadAction<{
        effectID: UUID;
        selector: TriggerEditorSelector<EffectWithID[]>;
      }>
    ) {
      const effects: EffectWithID[] = selector(slice);
      remove(effects, (effect) => effect.id === effectID);
    },

    setMatcherValue<M extends Matcher | MatcherWithContext>(
      slice: TriggerEditorState,
      {
        payload: { value, selector },
      }: PayloadAction<{
        value: string;
        selector: TriggerEditorSelector<M>;
      }>
    ) {
      const matcher = selector(slice);
      matcher.value.pattern = value;
    },

    appendNewMatcher<M extends Matcher | MatcherWithContext, F extends M[]>(
      slice: TriggerEditorState,
      {
        payload: { variant, selector },
      }: PayloadAction<{
        selector: TriggerEditorSelector<F>;
        variant: M['variant'];
      }>
    ) {
      const filter = selector(slice);
      filter.push({
        variant,
        value: { id: uuid(), pattern: '' },
      } as M);
    },

    deleteFilterMatcher(
      slice: TriggerEditorState,
      {
        payload: { index, selector },
      }: PayloadAction<{
        index: number;
        selector: TriggerEditorSelector<Filter | FilterWithContext>;
      }>
    ) {
      const matchers = selector(slice);
      matchers.splice(index, 1);
    },

    insertNewEffectOrTimerEffect(
      slice: TriggerEditorState,
      {
        payload: { triggerID, variant, index, seqSelector },
      }: PayloadAction<{
        variant: EffectVariant | TimerEffectVariant;
        index: number;
        triggerID: UUID;
        seqSelector: TriggerEditorSelector<EffectWithID[]>;
      }>
    ) {
      const effects = seqSelector(slice);
      const id = uuid();
      const effect: EffectWithID = {
        id,
        effect: isTimerEffectVariant(variant)
          ? {
              variant: 'ScopedTimerEffect',
              value: newTimerEffect(variant),
            }
          : newEffect(variant, triggerID),
      };
      effects.splice(index, 0, effect);
    },

    setTimerField<T extends TimerField>(
      slice: TriggerEditorState,
      {
        payload: { selector, field, value },
      }: PayloadAction<SetTimerFieldPayload<T>>
    ) {
      const timer: Timer = selector(slice);
      timer[field] = value;
    },

    setWaitUntilFilterMatchesDuration(
      slice: TriggerEditorState,
      {
        payload: { selector, duration },
      }: PayloadAction<{
        selector: TriggerEditorSelector<TimerEffectWaitUntilFilterMatchesType>;
        duration: Duration | null;
      }>
    ) {
      const waitUntilFilterMatches = selector(slice);
      waitUntilFilterMatches.value[1] = duration;
    },

    setWaitUntilSecondsRemainSeconds(
      slice: TriggerEditorState,
      {
        payload: { selector, seconds },
      }: PayloadAction<{
        seconds: number;
        selector: TriggerEditorSelector<TimerEffectWaitUntilSecondsRemainType>;
      }>
    ) {
      const waitUntilSecondsRemain = selector(slice);
      waitUntilSecondsRemain.value = seconds;
    },

    setCopyToClipboardTemplate(
      slice: TriggerEditorState,
      {
        payload: { tmpl, selector },
      }: PayloadAction<{
        tmpl: string;
        selector: TriggerEditorSelector<EffectVariantCopyToClipboard>;
      }>
    ) {
      const copyToClipboard = selector(slice);
      copyToClipboard.value = tmpl;
    },

    setSpeakTemplate(
      slice: TriggerEditorState,
      {
        payload: { tmpl, interrupt, selector },
      }: PayloadAction<{
        tmpl: string;
        interrupt: boolean;
        selector: TriggerEditorSelector<EffectVariantSpeak>;
      }>
    ) {
      const speak = selector(slice);
      speak.value = { tmpl, interrupt };
    },

    setOverlayMessageTemplate(
      slice: TriggerEditorState,
      {
        payload: { tmpl, selector },
      }: PayloadAction<{
        tmpl: string;
        selector: TriggerEditorSelector<EffectVariantOverlayMessage>;
      }>
    ) {
      const overlayMessage = selector(slice);
      overlayMessage.value = tmpl;
    },

    setPauseDuration(
      slice: TriggerEditorState,
      {
        payload: { millis, selector },
      }: PayloadAction<{
        millis: number;
        selector: TriggerEditorSelector<EffectVariantPause>;
      }>
    ) {
      const pause = selector(slice);
      pause.value = millis;
    },

    setTriggerTags(
      slice: TriggerEditorState,
      { payload: triggerTags }: PayloadAction<TriggerTag[]>
    ) {
      slice.draftTriggerTags = sortBy(triggerTags, (tag) =>
        tag.name.toUpperCase()
      );
    },

    setCommandTemplateSecurityCheck(
      slice: TriggerEditorState,
      {
        payload: { selector, cmdTmplSecCheck },
      }: PayloadAction<{
        selector: TriggerEditorSelector<EffectVariantRunSystemCommand>;
        cmdTmplSecCheck: CommandTemplateSecurityCheck;
      }>
    ) {
      const effect = selector(slice);
      effect.value = cmdTmplSecCheck;
    },

    setAudioFile(
      slice: TriggerEditorState,
      {
        payload: { path, selector },
      }: PayloadAction<{
        path: string | null;
        selector: TriggerEditorSelector<EffectVariantPlayAudioFile>;
      }>
    ) {
      const effect = selector(slice);
      effect.value = path;
    },

    setError(
      slice: TriggerEditorState,
      { payload: { id, error } }: PayloadAction<{ id: string; error: string }>
    ) {
      slice.errors[id] = error;
    },

    forgetError(
      slice: TriggerEditorState,
      { payload: id }: PayloadAction<string>
    ) {
      delete slice.errors[id];
    },
  },
});

export const {
  appendNewMatcher,
  cancelEditing,
  deleteEffect,
  deleteFilterMatcher,
  editNewTrigger,
  editTriggerDraft,
  forgetError,
  insertNewEffect,
  insertNewEffectOrTimerEffect,
  setAudioFile,
  setCommandTemplateSecurityCheck,
  setCopyToClipboardTemplate,
  setError,
  setMatcherValue,
  setOverlayMessageTemplate,
  setPauseDuration,
  setSpeakTemplate,
  setTimerField,
  setTriggerComment,
  setTriggerName,
  setTriggerTags,
  setWaitUntilFilterMatchesDuration,
  setWaitUntilSecondsRemainSeconds,
} = triggerEditorSlice.actions;

export default triggerEditorSlice.reducer;

export function triggerEditorSelector<T>(
  selector: TriggerEditorSelector<T>
): (state: MainRootState) => T {
  return (state: MainRootState) => selector(state[TRIGGER_EDITOR_SLICE]);
}

export const $draftTrigger = triggerEditorSelector(
  (slice: TriggerEditorState) => slice.draft!
);

export const $draftTriggerTags = triggerEditorSelector(
  ({ draftTriggerTags }) => draftTriggerTags
);

export const $draftParentPosition = triggerEditorSelector(
  ({ draftParentPosition }) => draftParentPosition
);

export const $$triggerDraftEffects = (slice: TriggerEditorState) =>
  slice.draft!.effects;

export const $$selectTriggerFilter = (slice: TriggerEditorState) =>
  slice.draft!.filter;

export const $selectTriggerFilter = triggerEditorSelector(
  $$selectTriggerFilter
);

export const $editorHasError = triggerEditorSelector((slice) =>
  some(Object.values(slice.errors), (bool) => bool)
);

export const $errorForID = (id: string) =>
  triggerEditorSelector((slice): string | undefined => slice.errors[id]);

function newEffect(variant: EffectVariant, triggerID: UUID): Effect {
  switch (variant) {
    case 'SpeakStop':
    case 'DoNothing':
      return { variant };
    case 'OverlayMessage':
    case 'CopyToClipboard':
      return { variant, value: '' };
    case 'Parallel':
    case 'Sequence':
      return { variant, value: [] };
    case 'Pause':
      return { variant, value: 0 };
    case 'PlayAudioFile':
      return { variant, value: null };
    case 'Speak':
      return { variant, value: { tmpl: '', interrupt: false } };
    case 'StartTimer':
      return {
        variant,
        value: {
          trigger_id: triggerID,
          name_tmpl: '',
          tags: [],
          duration: 0,
          start_policy: { variant: 'AlwaysStartNewTimer' },
          repeats: false,
          effects: [],
        },
      };
    case 'StartStopwatch':
      return { variant, value: { name: '', tags: [], effects: [] } };
    case 'RunSystemCommand':
      return {
        variant,
        value: {
          variant: 'Unapproved',
          value: { command: '', params: [], write_to_stdin: null },
        },
      };
    case 'ScopedTimerEffect':
      throw new Error('Tried to create a ScopedTimerEffect via newEffect');
  }
}

function newTimerEffect(variant: TimerEffectVariant): TimerEffect {
  switch (variant) {
    case 'ClearTimer':
    case 'HideTimer':
    case 'RestartTimer':
    case 'UnhideTimer':
    case 'WaitUntilFinished':
    case 'IncrementCounter':
    case 'DecrementCounter':
    case 'ResetCounter':
      return { variant };
    case 'WaitUntilFilterMatches':
      return { variant, value: [[], null] };
    case 'WaitUntilSecondsRemain':
      return { variant, value: 0 };
    case 'AddTag':
    case 'RemoveTag':
      return { variant, value: '' };
    default:
      throw 'UNRECOGNIZED TIMER EFFECT VARIANT ' + variant;
  }
}
