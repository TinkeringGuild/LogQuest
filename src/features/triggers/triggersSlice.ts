import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { MainRootState } from '../../MainStore';
import { DataDelta } from '../../generated/DataDelta';
import { TriggerGroup } from '../../generated/TriggerGroup';
import { TriggerGroupDescendant } from '../../generated/TriggerGroupDescendant';
import { TriggerIndex } from '../../generated/TriggerIndex';
import { UUID } from '../../generated/UUID';
import * as deltas from './deltas';

export const TRIGGERS_SLICE = 'triggers';

interface TriggersState {
  index: TriggerIndex;
  activeTriggerTagID: UUID | null;
}

export type TriggersSelector<T> = (slice: TriggersState) => T;

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
        if (variant === 'TriggerSaved') {
          deltas[variant](state.index, value);
        } else if (variant === 'TriggerDeleted') {
          deltas[variant](state.index, value);
        } else if (variant === 'TriggerGroupSaved') {
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
        } else if (variant === 'TriggerTagTriggersChanged') {
          deltas[variant](state.index, value);
        } else if (variant === 'TriggerGroupDeleted') {
          deltas[variant](state.index, value);
        } else {
          throw new Error('UNIMPLEMENTED DELTA TYPE: ' + variant);
        }
      });
    },
  },
});

export const { initTriggers, activateTriggerTagID, applyDeltas } =
  triggersSlice.actions;

export default triggersSlice.reducer;

export function triggersSelector<T>(
  selector: TriggersSelector<T>
): (state: MainRootState) => T {
  return (state: MainRootState) => selector(state[TRIGGERS_SLICE]);
}

export const $topLevel = triggersSelector(
  ({ index: { top_level } }) => top_level
);

export const $trigger = (triggerID: UUID) => {
  return triggersSelector(({ index: { triggers } }) => triggers[triggerID]);
};

export const $triggerGroup = (groupID: UUID) => {
  return triggersSelector((slice) => slice.index.groups[groupID]);
};

export const $triggerGroups = triggersSelector(
  ({ index: { groups } }) => groups
);

export const $triggerTags = triggersSelector(
  ({ index: { trigger_tags } }) => trigger_tags
);

export const $activeTriggerTagID = triggersSelector(
  ({ activeTriggerTagID }) => activeTriggerTagID
);

export const $activeTriggerTag = triggersSelector(
  ({ activeTriggerTagID, index: { trigger_tags } }) =>
    activeTriggerTagID ? trigger_tags[activeTriggerTagID] : null
);

export const $$triggerTagsHavingTrigger = (triggerID: UUID) => {
  return ({ index: { trigger_tags } }: TriggersState) => {
    return Object.values(trigger_tags).filter((tag) =>
      tag.triggers.includes(triggerID)
    );
  };
};

export const $triggerTagsHavingTrigger = (triggerID: UUID) => {
  return triggersSelector($$triggerTagsHavingTrigger(triggerID));
};

export const $groupsUptoGroup = (groupID: UUID | null) =>
  triggersSelector((slice) => {
    if (!groupID) {
      return [];
    }
    const groups = slice.index.groups;
    const deepest: TriggerGroup = groups[groupID];
    let parent_id: UUID | null = deepest.parent_id;
    const path: TriggerGroup[] = [deepest];
    while (parent_id) {
      const parent = groups[parent_id];
      parent_id = parent.parent_id;
      path.unshift(parent);
    }
    return path;
  });

export const $positionOfTrigger = (triggerID: UUID) =>
  triggersSelector((slice) => {
    const trigger = slice.index.triggers[triggerID];
    const peers: TriggerGroupDescendant[] = trigger.parent_id
      ? slice.index.groups[trigger.parent_id].children
      : slice.index.top_level;
    return peers.findIndex(
      (tgd) => tgd.variant === 'T' && tgd.value === triggerID
    );
  });
