import { createSlice } from '@reduxjs/toolkit';

import { TriggerRoot } from '../../generated/TriggerRoot';
import { LQ_VERSION } from '../../generated/constants';
import { LogQuestVersion } from '../../generated/LogQuestVersion';

export const TRIGGERS_SLICE = 'triggers';

interface TriggersState {
  root: TriggerRoot;
}

const INITIAL_TRIGGERS_STATE: TriggersState = {
  root: {
    groups: [],
    log_quest_version: LQ_VERSION as LogQuestVersion,
  },
};

const triggersSlice = createSlice({
  name: TRIGGERS_SLICE,
  initialState: INITIAL_TRIGGERS_STATE,
  reducers: {
    initTriggers(
      state: TriggersState,
      { payload: root }: { payload: TriggerRoot }
    ) {
      state.root = root;
    },
  },
});
export const { initTriggers } = triggersSlice.actions;
export default triggersSlice.reducer;

export const $triggerGroups = ({
  [TRIGGERS_SLICE]: triggers,
}: {
  [TRIGGERS_SLICE]: TriggersState;
}) => triggers.root.groups;
