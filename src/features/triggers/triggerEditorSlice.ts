import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { pullAt, remove, sortBy } from 'lodash';

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
import { TriggerTag } from '../../generated/TriggerTag';
import { UUID } from '../../generated/UUID';
import { MainRootState } from '../../MainStore';

export const TRIGGER_EDITOR_SLICE = 'trigger-editor';

export type TriggerEditorState = {
  draft: Trigger | null;
  draftTriggerTags: TriggerTag[];
};

const INITIAL_TRIGGER_EDITOR_STATE = {
  draft: null,
  draftTriggerTags: [],
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
      slice: TriggerEditorState,
      {
        payload: { trigger, triggerTags },
      }: PayloadAction<{ trigger: Trigger; triggerTags: TriggerTag[] }>
    ) {
      slice.draft = trigger;
      slice.draftTriggerTags = sortBy(triggerTags, (tag) => tag.name);
    },

    cancelEditing(slice: TriggerEditorState) {
      slice.draft = null;
      slice.draftTriggerTags = [];
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

    setMatcherValue(
      slice: TriggerEditorState,
      {
        payload: { value, selector },
      }: PayloadAction<{
        value: string;
        selector: TriggerEditorSelector<Matcher | MatcherWithContext>;
      }>
    ) {
      const matcher = selector(slice);
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

    setTriggerTags(
      slice: TriggerEditorState,
      { payload: triggerTags }: PayloadAction<TriggerTag[]>
    ) {
      slice.draftTriggerTags = sortBy(triggerTags, (tag) => tag.name);
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
  setTriggerTags,
  setWaitUntilFilterMatchesDuration,
} = triggerEditorSlice.actions;

export default triggerEditorSlice.reducer;

export function triggerEditorSelector<T>(
  selector: TriggerEditorSelector<T>
): (state: MainRootState) => T {
  return (state: MainRootState) => selector(state[TRIGGER_EDITOR_SLICE]);
}

export const $draftTrigger = triggerEditorSelector(
  (slice: TriggerEditorState) => slice.draft
);

export const $draftTriggerTags = triggerEditorSelector(
  ({ draftTriggerTags }) => draftTriggerTags
);

export const $$triggerDraftEffects = (slice: TriggerEditorState) =>
  slice.draft!.effects;

export const $$selectTriggerFilter = (slice: TriggerEditorState) =>
  slice.draft!.filter;

export const $selectTriggerFilter = triggerEditorSelector(
  $$selectTriggerFilter
);
