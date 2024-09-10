import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { pullAt, remove } from 'lodash';

import { Duration } from '../../generated/Duration';
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
};

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

const INITIAL_EDITOR_STATE = {
  draft: null,
} satisfies EditorState;

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
      const timerEffect = selector(slice);
      timerEffect.value[1] = duration;
    },
  },
});

export const {
  appendNewMatcher,
  deleteEffect,
  deleteFilterMatcher,
  editTriggerDraft,
  setMatcherValue,
  setTimerField,
  setTriggerName,
  setTriggerComment,
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
