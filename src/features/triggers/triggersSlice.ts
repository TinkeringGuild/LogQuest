import { createSelector, createSlice, PayloadAction } from '@reduxjs/toolkit';

import { MainRootState } from '../../MainStore';
import { DataDelta } from '../../generated/DataDelta';
import { TriggerGroup } from '../../generated/TriggerGroup';
import { TriggerGroupDescendant } from '../../generated/TriggerGroupDescendant';
import { TriggerIndex } from '../../generated/TriggerIndex';
import { UUID } from '../../generated/UUID';
import * as deltas from './deltas';
import { $PARAM } from '../common';

export const TRIGGERS_SLICE = 'triggers';

interface TriggersState {
  index: TriggerIndex;
  activeTriggerTagID: UUID | null;
  filter: null | {
    text: string;
    triggerIDs: Set<UUID>;
    groupIDs: Set<UUID>;
  };
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
  filter: null,
};

const WHITESPACE_REGEX = /\s+/g;

const triggersSlice = createSlice({
  name: TRIGGERS_SLICE,
  initialState: INITIAL_TRIGGERS_STATE,
  reducers: {
    initTriggers(
      slice: TriggersState,
      { payload: index }: PayloadAction<TriggerIndex>
    ) {
      slice.index = index;
    },

    activateTriggerTagID(
      slice: TriggersState,
      { payload: triggerTagID }: PayloadAction<UUID | null>
    ) {
      slice.activeTriggerTagID = triggerTagID;
    },

    search(slice: TriggersState, { payload: text }: PayloadAction<string>) {
      const textTrimmed = text.trim();

      if (textTrimmed === '') {
        slice.filter = {
          text: text,
          triggerIDs: new Set(),
          groupIDs: new Set(),
        };
        return;
      }

      const formattedSearch = textTrimmed
        .toUpperCase()
        .replace(WHITESPACE_REGEX, '');

      const filterByName = ({ name }: { name: string }) =>
        name
          .toUpperCase()
          .replace(WHITESPACE_REGEX, '')
          .includes(formattedSearch);

      const mapIDs = ({ id }: { id: UUID }) => id;

      const triggers = Object.values(slice.index.triggers).filter(filterByName);
      const triggerIDs = new Set(triggers.map(mapIDs));

      const groups = Object.values(slice.index.groups).filter(filterByName);
      const groupIDs = new Set<UUID>(groups.map(mapIDs));

      for (const trigger of triggers) {
        let parentID = trigger.parent_id;
        while (parentID) {
          groupIDs.add(parentID);
          parentID = slice.index.groups[parentID]?.parent_id;
        }
      }

      slice.filter = {
        text,
        triggerIDs,
        groupIDs,
      };
    },

    clearSearch(slice: TriggersState) {
      slice.filter = null;
    },

    applyDeltas(state: TriggersState, { payload }: PayloadAction<DataDelta[]>) {
      // console.log('APPLYING DELTAS:', payload);
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
        } else if (variant === 'TriggerTagRenamed') {
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

export const {
  initTriggers,
  activateTriggerTagID,
  applyDeltas,
  search,
  clearSearch,
} = triggersSlice.actions;

export default triggersSlice.reducer;

const createTriggersSelector = createSelector.withTypes<TriggersState>();

export function triggersSelector<T>(
  selector: TriggersSelector<T>
): (state: MainRootState) => T {
  return (state: MainRootState) => selector(state[TRIGGERS_SLICE]);
}

const $$topLevel = ({ index: { top_level } }: TriggersState) => top_level;
export const $topLevel = triggersSelector($$topLevel);

const $$triggers = ({ index: { triggers } }: TriggersState) => triggers;
export const $triggers = triggersSelector($$triggers);

const triggerIDParam = $PARAM<UUID>();
const selectTrigger = createSelector(
  [$triggers, triggerIDParam],
  (triggers, triggerID) => triggers[triggerID]
);

export const $trigger = (triggerID: UUID) => (state: MainRootState) =>
  selectTrigger(state, triggerID);

const $$triggerGroups = ({ index: { groups } }: TriggersState) => groups;
export const $triggerGroups = triggersSelector($$triggerGroups);

const groupIDParam = $PARAM<UUID>();
const selectGroup = createSelector(
  [$triggerGroups, groupIDParam],
  (groups, group_ID) => groups[group_ID]
);
export const $triggerGroup = (groupID: UUID) => (state: MainRootState) =>
  selectGroup(state, groupID);

export const $$triggerTags = ({ index: { trigger_tags } }: TriggersState) =>
  trigger_tags;

export const $triggerTags = triggersSelector($$triggerTags);

export const $activeTriggerTagID = triggersSelector(
  ({ activeTriggerTagID }) => activeTriggerTagID
);

export const $activeTriggerTag = triggersSelector(
  ({ activeTriggerTagID, index: { trigger_tags } }) =>
    activeTriggerTagID ? trigger_tags[activeTriggerTagID] : null
);

const selectTriggerTagsHavingTrigger = createTriggersSelector(
  [$$triggerTags, triggerIDParam],
  (triggerTags, triggerID) =>
    Object.values(triggerTags).filter((tag) => tag.triggers.includes(triggerID))
);
export const $$triggerTagsHavingTrigger =
  (triggerID: UUID) => (slice: TriggersState) =>
    selectTriggerTagsHavingTrigger(slice, triggerID);

export const $triggerTagsHavingTrigger = (triggerID: UUID) =>
  triggersSelector($$triggerTagsHavingTrigger(triggerID));

export const $groups = triggersSelector((slice) => slice.index.groups);

export const paramGroupID = $PARAM<UUID | null>();
export const selectGroupsUptoGroup = createSelector(
  [$groups, paramGroupID],
  (groups, groupID) => {
    if (!groupID) {
      return [];
    }
    const deepest: TriggerGroup = groups[groupID];
    let parent_id: UUID | null = deepest.parent_id;
    const path: TriggerGroup[] = [deepest];
    while (parent_id) {
      const parent = groups[parent_id];
      path.unshift(parent);
      parent_id = parent.parent_id;
    }
    return path;
  }
);

export const $groupsUptoGroup =
  (groupID: UUID | null) => (state: MainRootState) =>
    selectGroupsUptoGroup(state, groupID);

const typeAndIDParam = $PARAM<{ trigger: UUID } | { group: UUID }>();
const selectPositionOf = createTriggersSelector(
  [$$triggers, $$triggerGroups, $$topLevel, typeAndIDParam],
  (triggers, groups, topLevel, typeAndID) => {
    const [tgdVariant, triggerOrGroup] =
      'trigger' in typeAndID
        ? ['T', triggers[typeAndID.trigger]]
        : ['G', groups[typeAndID.group]];

    const peers: TriggerGroupDescendant[] = triggerOrGroup.parent_id
      ? groups[triggerOrGroup.parent_id].children
      : topLevel;

    return peers.findIndex(
      (tgd) => tgd.variant === tgdVariant && tgd.value === triggerOrGroup.id
    );
  }
);

export const $positionOf = (typeAndID: { trigger: UUID } | { group: UUID }) =>
  triggersSelector((slice) => selectPositionOf(slice, typeAndID));

export const $filter = triggersSelector(({ filter }) => filter);
