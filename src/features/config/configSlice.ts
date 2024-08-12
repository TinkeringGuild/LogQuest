import { createSlice } from '@reduxjs/toolkit';

import { LogQuestConfig } from '../../generated/LogQuestConfig';
import { RootState } from '../../store';

export const configInitialState: LogQuestConfig = {
  everquest_directory: null,
};

const configSlice = createSlice({
  name: 'config',
  initialState: configInitialState,
  reducers: {
    initConfig: (_state, { payload }) => payload,
  },
});

export default configSlice.reducer;

export const { initConfig } = configSlice.actions;

export const selectEQDirIsBlank = (state: RootState) =>
  !state.config.everquest_directory; // empty strings are falsy in JS too
