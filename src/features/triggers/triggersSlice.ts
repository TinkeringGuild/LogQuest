import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { TriggerRoot } from '../../generated/TriggerRoot';
import { LQ_VERSION } from '../../generated/constants';
import { LogQuestVersion } from '../../generated/LogQuestVersion';
import { UUID } from '../../generated/UUID';
import { TriggerGroup } from '../../generated/TriggerGroup';
import { Trigger } from '../../generated/Trigger';

export const TRIGGERS_SLICE = 'triggers';

interface TriggersState {
  root: TriggerRoot;
  activatedTriggerID: UUID | null;
}

const INITIAL_TRIGGERS_STATE: TriggersState = {
  root: {
    groups: [],
    log_quest_version: LQ_VERSION as LogQuestVersion,
  },
  activatedTriggerID: null,
};

const triggersSlice = createSlice({
  name: TRIGGERS_SLICE,
  initialState: INITIAL_TRIGGERS_STATE,
  reducers: {
    initTriggers(
      state: TriggersState,
      { payload: root }: PayloadAction<TriggerRoot>
    ) {
      state.root = root;
    },

    activateTriggerID(
      state: TriggersState,
      { payload: triggerID }: PayloadAction<UUID>
    ) {
      if (state.activatedTriggerID === triggerID) {
        state.activatedTriggerID = null;
      } else {
        state.activatedTriggerID = triggerID;
      }
    },

    setTriggerEnabled(
      state: TriggersState,
      {
        payload: { triggerID, enabled },
      }: PayloadAction<{ triggerID: UUID; enabled: boolean }>
    ) {
      for (const group of state.root.groups) {
        const search = $triggerInGroupWithID(group, triggerID);
        if (search !== null) {
          search.enabled = enabled;
          return;
        }
      }
    },
  },
});
export const { initTriggers, activateTriggerID, setTriggerEnabled } =
  triggersSlice.actions;
export default triggersSlice.reducer;

const $triggerInGroupWithID: (
  group: TriggerGroup,
  triggerID: UUID
) => Trigger | null = (group, triggerID) => {
  for (const tgd of group.children) {
    if ('T' in tgd) {
      if (tgd.T.id == triggerID) {
        return tgd.T;
      }
    } else if ('TG' in tgd) {
      const search: Trigger | null = $triggerInGroupWithID(tgd.TG, triggerID);
      if (search !== null) {
        return search;
      }
    }
  }
  return null;
};

export const $triggerGroups = ({
  [TRIGGERS_SLICE]: triggers,
}: {
  [TRIGGERS_SLICE]: TriggersState;
}) => triggers.root.groups;

export const $triggerWithID = (triggerID: UUID) => {
  return (state: { [TRIGGERS_SLICE]: TriggersState }) => {
    const triggers = state[TRIGGERS_SLICE];
    for (const group of triggers.root.groups) {
      const search = $triggerInGroupWithID(group, triggerID);
      if (search !== null) {
        return search;
      }
    }
    return null;
  };
};

export const $currentTriggerID = ({
  [TRIGGERS_SLICE]: { activatedTriggerID },
}: {
  [TRIGGERS_SLICE]: TriggersState;
}) => activatedTriggerID;

export const $currentTrigger = (state: { [TRIGGERS_SLICE]: TriggersState }) => {
  const triggerID = state[TRIGGERS_SLICE].activatedTriggerID;
  if (triggerID === null) {
    return null;
  }
  return $triggerWithID(triggerID)(state);
};
