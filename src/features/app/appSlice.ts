import { createSlice } from '@reduxjs/toolkit';

import { MainDispatch, MainRootState } from '../../MainStore';

export const APP_SLICE = 'app';

export type MODE =
  | 'overview'
  | 'triggers'
  | 'overlay'
  | 'settings'
  | 'help'
  | 'about';

export interface AppState {
  bootstrapped: boolean;
  currentMode: MODE;
  isLoading: boolean;
}

const INITIAL_APP_STATE: AppState = {
  bootstrapped: false,
  currentMode: 'triggers',
  isLoading: false,
};

const appSlice = createSlice({
  name: APP_SLICE,
  initialState: INITIAL_APP_STATE,
  reducers: {
    bootstrapHasLoaded(state: AppState) {
      state.bootstrapped = true;
    },
    navigateTo(state: AppState, { payload: mode }: { payload: MODE }) {
      state.currentMode = mode;
    },
    enterLoadingState(state: AppState) {
      state.isLoading = true;
    },
    exitLoadingState(state: AppState) {
      state.isLoading = false;
    },
  },
});
export default appSlice.reducer;
export const {
  bootstrapHasLoaded,
  navigateTo,
  enterLoadingState,
  exitLoadingState,
} = appSlice.actions;

export const $isBootstrapped = ({ [APP_SLICE]: app }: MainRootState) =>
  app.bootstrapped;

export const $currentMode = ({ [APP_SLICE]: app }: { [APP_SLICE]: AppState }) =>
  app.currentMode;

export const $isLoading = ({ [APP_SLICE]: app }: { [APP_SLICE]: AppState }) =>
  app.isLoading || !app.bootstrapped;
