import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { DataDelta } from '../../generated/DataDelta';
import { Effect } from '../../generated/Effect';
import { TriggerIndex } from '../../generated/TriggerIndex';
import { UUID } from '../../generated/UUID';
import * as deltas from './deltas';

export const TRIGGERS_SLICE = 'triggers';

interface TriggersState {
  index: TriggerIndex;
  activeTriggerTagID: UUID | null;
}

const INITIAL_TRIGGERS_STATE: TriggersState = {
  index: {
    triggers: {},
    groups: {},
    top_level: [],
    trigger_tags: {},
  },
  activeTriggerTagID: null,
};

const triggersSlice = createSlice({
  name: TRIGGERS_SLICE,
  initialState: INITIAL_TRIGGERS_STATE,
  reducers: {
    initTriggers(
      state: TriggersState,
      { payload: index }: PayloadAction<TriggerIndex>
    ) {
      state.index = index;
    },

    activateTriggerTagID(
      state: TriggersState,
      { payload: triggerTagID }: PayloadAction<UUID | null>
    ) {
      state.activeTriggerTagID = triggerTagID;
    },

    applyDeltas(state: TriggersState, { payload }: PayloadAction<DataDelta[]>) {
      payload.forEach(({ variant, value }) => {
        if (variant === 'TriggerUpdated') {
          deltas[variant](state.index, value);
        } else if (variant === 'TriggerGroupCreated') {
          deltas[variant](state.index, value);
        } else if (variant === 'TriggerGroupChildrenChanged') {
          deltas[variant](state.index, value);
        } else if (variant === 'TopLevelChanged') {
          deltas[variant](state.index, value);
        } else if (variant === 'TriggerTagged') {
          deltas[variant](state.index, value);
        } else if (variant === 'TriggerUntagged') {
          deltas[variant](state.index, value);
        } else if (variant === 'TriggerTagCreated') {
          deltas[variant](state.index, value);
        } else if (variant === 'TriggerTagDeleted') {
          deltas[variant](state.index, value);
        } else {
          throw new Error('UNIMPLEMENTED DELTA TYPE: ' + variant);
        }
      });
    },

    updateTriggerEffect(
      state: TriggersState,
      action: PayloadAction<{
        triggerID: UUID;
        effectID: UUID;
        mutation: (effect: Effect) => void;
      }>
    ) {
      const { triggerID, effectID, mutation } = action.payload;
      const trigger = $trigger(triggerID)({
        [TRIGGERS_SLICE]: state,
      });
      if (trigger) {
        const effect = trigger.effects.find((e) => e.id === effectID);
        if (effect) {
          mutation(effect.inner);
        }
      }
    },
  },
});

export const {
  initTriggers,
  activateTriggerTagID,
  applyDeltas,
  updateTriggerEffect,
} = triggersSlice.actions;

export default triggersSlice.reducer;

export const $topLevel = ({
  [TRIGGERS_SLICE]: {
    index: { top_level },
  },
}: {
  [TRIGGERS_SLICE]: TriggersState;
}) => top_level;

export const $trigger = (triggerID: UUID) => {
  return (state: { [TRIGGERS_SLICE]: TriggersState }) => {
    const triggers = state[TRIGGERS_SLICE];
    return triggers.index.triggers[triggerID];
  };
};

export const $triggerGroup = (groupID: UUID) => {
  return (state: { [TRIGGERS_SLICE]: TriggersState }) => {
    const triggers = state[TRIGGERS_SLICE];
    return triggers.index.groups[groupID];
  };
};

export const $triggerGroups = ({
  [TRIGGERS_SLICE]: triggers,
}: {
  [TRIGGERS_SLICE]: TriggersState;
}) => triggers.index.groups;

export const $triggerTags = ({
  [TRIGGERS_SLICE]: triggers,
}: {
  [TRIGGERS_SLICE]: TriggersState;
}) => triggers.index.trigger_tags;

export const $activeTriggerTagID = ({
  [TRIGGERS_SLICE]: { activeTriggerTagID },
}: {
  [TRIGGERS_SLICE]: TriggersState;
}) => activeTriggerTagID;

export const $activeTriggerTag = ({
  [TRIGGERS_SLICE]: triggers,
}: {
  [TRIGGERS_SLICE]: TriggersState;
}) =>
  triggers.activeTriggerTagID
    ? triggers.index.trigger_tags[triggers.activeTriggerTagID]
    : null;
