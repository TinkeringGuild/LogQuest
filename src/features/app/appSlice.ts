import { createSlice } from '@reduxjs/toolkit';

import { RootState } from '../../store';
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

export const hasBootstrapped = (state: RootState) => state.app.bootstrapped;
