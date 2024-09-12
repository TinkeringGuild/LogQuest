import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { pullAt, remove } from 'lodash';

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
import { UUID } from '../../generated/UUID';
import { MainRootState } from '../../MainStore';

export const TRIGGER_EDITOR_SLICE = 'trigger-editor';

export type TriggerEditorState = {
  draft: Trigger | null;
  disabled: boolean;
};

const INITIAL_TRIGGER_EDITOR_STATE = {
  draft: null,
  disabled: false,
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

export type EffectVariantPause = Extract<Effect, { variant: 'Pause' }>;

const triggerEditorSlice = createSlice({
  name: TRIGGER_EDITOR_SLICE,
  initialState: INITIAL_TRIGGER_EDITOR_STATE,

  reducers: {
    editTriggerDraft(
      state: TriggerEditorState,
      { payload: trigger }: PayloadAction<Trigger>
    ) {
      state.draft = trigger;
    },

    cancelEditing(state: TriggerEditorState) {
      state.draft = null;
    },

    setTriggerName(
      slice: TriggerEditorState,
      { payload: name }: PayloadAction<string>
    ) {
      slice.draft!.name = name;
    },

    setTriggerComment(
      state: TriggerEditorState,
      { payload: comment }: PayloadAction<string>
    ) {
      state.draft!.comment = comment;
    },

    deleteEffect(
      state: TriggerEditorState,
      {
        payload: { effectID, selector },
      }: PayloadAction<{
        effectID: UUID;
        selector: TriggerEditorSelector<EffectWithID[]>;
      }>
    ) {
      const effects: EffectWithID[] = selector(state);
      remove(effects, (effect) => effect.id === effectID);
    },

    setMatcherValue(
      state: TriggerEditorState,
      {
        payload: { value, selector },
      }: PayloadAction<{
        value: string;
        selector: TriggerEditorSelector<Matcher | MatcherWithContext>;
      }>
    ) {
      const matcher = selector(state);
      matcher.value = value;
    },

    appendNewMatcher<M extends Matcher | MatcherWithContext, F extends M[]>(
      slice: TriggerEditorState,
      {
        payload: { matcherVariant, selector },
      }: PayloadAction<{
        selector: TriggerEditorSelector<F>;
        matcherVariant: M['variant'];
      }>
    ) {
      const filter = selector(slice);
      filter.push({ variant: matcherVariant, value: '' } as M);
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
      pullAt(matchers, index);
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
        duration: Duration;
      }>
    ) {
      const waitUntilFilterMatches = selector(slice);
      waitUntilFilterMatches.value[1] = duration;
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
  },
});

export const {
  appendNewMatcher,
  cancelEditing,
  deleteEffect,
  deleteFilterMatcher,
  editTriggerDraft,
  setCopyToClipboardTemplate,
  setMatcherValue,
  setOverlayMessageTemplate,
  setPauseDuration,
  setSpeakTemplate,
  setTimerField,
  setTriggerComment,
  setTriggerName,
  setWaitUntilFilterMatchesDuration,
} = triggerEditorSlice.actions;

export default triggerEditorSlice.reducer;

export function triggerEditorSelector<T>(
  selector: TriggerEditorSelector<T>
): (state: MainRootState) => T {
  return (state: MainRootState) => selector(state[TRIGGER_EDITOR_SLICE]);
}

export const $triggerDraft = triggerEditorSelector(
  (slice: TriggerEditorState) => slice.draft
);

export const $$triggerDraftEffects = (slice: TriggerEditorState) =>
  slice.draft!.effects;

export const $$selectTriggerFilter = (slice: TriggerEditorState) =>
  slice.draft!.filter;

export const $selectTriggerFilter = triggerEditorSelector(
  $$selectTriggerFilter
);

export const $editingDisabled = triggerEditorSelector(
  (slice: TriggerEditorState) => slice.disabled
);
