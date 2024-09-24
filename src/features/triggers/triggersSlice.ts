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

export const $positionOf = (typeAndID: { trigger: UUID } | { group: UUID }) =>
  triggersSelector((slice) => {
    const [tgdVariant, triggerOrGroup] =
      'trigger' in typeAndID
        ? ['T', slice.index.triggers[typeAndID.trigger]]
        : ['G', slice.index.groups[typeAndID.group]];

    const peers: TriggerGroupDescendant[] = triggerOrGroup.parent_id
      ? slice.index.groups[triggerOrGroup.parent_id].children
      : slice.index.top_level;

    return peers.findIndex(
      (tgd) => tgd.variant === tgdVariant && tgd.value === triggerOrGroup.id
    );
  });

export const $filter = triggersSelector(({ filter }) => filter);
