import { createSlice } from '@reduxjs/toolkit';

import { MainRootState } from '../../MainStore';
import { AppState } from '../../types';

const INITIAL_APP_STATE: AppState = {
  bootstrapped: false,
};

const appSlice = createSlice({
  name: 'app',
  initialState: INITIAL_APP_STATE,
  reducers: {
    bootstrapHasLoaded: (state: AppState) => {
      state.bootstrapped = true;
      return state;
    },
  },
});

export default appSlice.reducer;

export const { bootstrapHasLoaded } = appSlice.actions;

export const hasBootstrapped = (state: MainRootState) => state.app.bootstrapped;
