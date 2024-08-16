import { createSlice } from '@reduxjs/toolkit';

import { LogQuestConfig } from '../../generated/LogQuestConfig';

export const CONFIG_SLICE = 'config';

export const configInitialState: LogQuestConfig = {
  everquest_directory: null,
};

const configSlice = createSlice({
  name: CONFIG_SLICE,
  initialState: configInitialState,
  reducers: {
    initConfig: (_state, { payload }) => payload,
  },
});
export default configSlice.reducer;
export const { initConfig } = configSlice.actions;

export const $eqDirBlank = ({
  [CONFIG_SLICE]: config,
}: {
  [CONFIG_SLICE]: LogQuestConfig;
}) => !config.everquest_directory; // empty strings are falsy in JS too
