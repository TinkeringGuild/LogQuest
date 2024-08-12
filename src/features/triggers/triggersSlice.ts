import { createSlice } from '@reduxjs/toolkit';

import { TriggerRoot } from '../../generated/TriggerRoot';
import { RootState } from '../../store';

const INITIAL_TRIGGERS_STATE: TriggerRoot = {
  log_quest_version: [0, 0, 0], // TODO: Use vite-plugin-package-config to get this initial value from package.json?
  groups: [],
};

const triggersSlice = createSlice({
  name: 'triggers',
  initialState: INITIAL_TRIGGERS_STATE,
  reducers: {
    initTriggers: (_state, { payload }) => payload,
  },
});

export const selectTriggerGroups = (state: RootState) => state.triggers.groups;

export const { initTriggers } = triggersSlice.actions;

export default triggersSlice.reducer;
