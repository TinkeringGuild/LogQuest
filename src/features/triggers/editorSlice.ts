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

export const EDITOR_SLICE = 'trigger-editor';

export type EditorState = {
  draft: Trigger | null;
  disabled: boolean;
};

const INITIAL_EDITOR_STATE = {
  draft: null,
  disabled: false,
} satisfies EditorState;

export type EditorSelector<T> = (slice: EditorState) => T;

type TimerField = keyof Timer;
type TimerFieldValue<T extends TimerField> = Timer[T];
interface SetTimerFieldPayload<T extends TimerField> {
  field: T;
  value: TimerFieldValue<T>;
  selector: EditorSelector<Timer>;
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

const editorSlice = createSlice({
  name: EDITOR_SLICE,
  initialState: INITIAL_EDITOR_STATE,

  reducers: {
    editTriggerDraft(
      state: EditorState,
      { payload: trigger }: PayloadAction<Trigger>
    ) {
      state.draft = trigger;
    },

    cancelEditing(state: EditorState) {
      state.draft = null;
    },

    setTriggerName(
      slice: EditorState,
      { payload: name }: PayloadAction<string>
    ) {
      slice.draft!.name = name;
    },

    setTriggerComment(
      state: EditorState,
      { payload: comment }: PayloadAction<string>
    ) {
      state.draft!.comment = comment;
    },

    deleteEffect(
      state: EditorState,
      {
        payload: { effectID, selector },
      }: PayloadAction<{
        effectID: UUID;
        selector: EditorSelector<EffectWithID[]>;
      }>
    ) {
      const effects: EffectWithID[] = selector(state);
      remove(effects, (effect) => effect.id === effectID);
    },

    setMatcherValue(
      state: EditorState,
      {
        payload: { value, selector },
      }: PayloadAction<{
        value: string;
        selector: EditorSelector<Matcher | MatcherWithContext>;
      }>
    ) {
      const matcher = selector(state);
      matcher.value = value;
    },

    appendNewMatcher<M extends Matcher | MatcherWithContext, F extends M[]>(
      slice: EditorState,
      {
        payload: { matcherVariant, selector },
      }: PayloadAction<{
        selector: EditorSelector<F>;
        matcherVariant: M['variant'];
      }>
    ) {
      const filter = selector(slice);
      filter.push({ variant: matcherVariant, value: '' } as M);
    },

    deleteFilterMatcher(
      slice: EditorState,
      {
        payload: { index, selector },
      }: PayloadAction<{
        index: number;
        selector: EditorSelector<Filter | FilterWithContext>;
      }>
    ) {
      const matchers = selector(slice);
      pullAt(matchers, index);
    },

    setTimerField<T extends TimerField>(
      slice: EditorState,
      {
        payload: { selector, field, value },
      }: PayloadAction<SetTimerFieldPayload<T>>
    ) {
      const timer: Timer = selector(slice);
      timer[field] = value;
    },

    setWaitUntilFilterMatchesDuration(
      slice: EditorState,
      {
        payload: { selector, duration },
      }: PayloadAction<{
        selector: EditorSelector<TimerEffectWaitUntilFilterMatchesType>;
        duration: Duration;
      }>
    ) {
      const waitUntilFilterMatches = selector(slice);
      waitUntilFilterMatches.value[1] = duration;
    },

    setCopyToClipboardTemplate(
      slice: EditorState,
      {
        payload: { tmpl, selector },
      }: PayloadAction<{
        tmpl: string;
        selector: EditorSelector<EffectVariantCopyToClipboard>;
      }>
    ) {
      const copyToClipboard = selector(slice);
      copyToClipboard.value = tmpl;
    },

    setSpeakTemplate(
      slice: EditorState,
      {
        payload: { tmpl, interrupt, selector },
      }: PayloadAction<{
        tmpl: string;
        interrupt: boolean;
        selector: EditorSelector<EffectVariantSpeak>;
      }>
    ) {
      const speak = selector(slice);
      speak.value = { tmpl, interrupt };
    },

    setOverlayMessageTemplate(
      slice: EditorState,
      {
        payload: { tmpl, selector },
      }: PayloadAction<{
        tmpl: string;
        selector: EditorSelector<EffectVariantOverlayMessage>;
      }>
    ) {
      const overlayMessage = selector(slice);
      overlayMessage.value = tmpl;
    },

    setPauseDuration(
      slice: EditorState,
      {
        payload: { millis, selector },
      }: PayloadAction<{
        millis: number;
        selector: EditorSelector<EffectVariantPause>;
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
} = editorSlice.actions;

export default editorSlice.reducer;

export function editorSelector<T>(
  selector: EditorSelector<T>
): (state: MainRootState) => T {
  return (state: MainRootState) => selector(state[EDITOR_SLICE]);
}

export const $triggerDraft = editorSelector(
  (slice: EditorState) => slice.draft
);

export const $$triggerDraftEffects = (slice: EditorState) =>
  slice.draft!.effects;

export const $$selectTriggerFilter = (slice: EditorState) =>
  slice.draft!.filter;

export const $selectTriggerFilter = editorSelector($$selectTriggerFilter);

export const $editingDisabled = editorSelector(
  (slice: EditorState) => slice.disabled
);
