import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { MainRootState } from '../../MainStore';
import { ProgressUpdate } from '../../generated/ProgressUpdate';
import { seqFromProgressUpdate } from '../../util';

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
  progressUpdate: ProgressUpdate | null;
}

const INITIAL_APP_STATE: AppState = {
  bootstrapped: false,
  currentMode: 'triggers',
  isLoading: false,
  progressUpdate: null,
};

const appSlice = createSlice({
  name: APP_SLICE,
  initialState: INITIAL_APP_STATE,
  reducers: {
    bootstrapHasLoaded(state: AppState) {
      state.bootstrapped = true;
    },
    navigateTo(state: AppState, { payload: mode }: PayloadAction<MODE>) {
      state.currentMode = mode;
    },
    enterLoadingState(state: AppState) {
      state.isLoading = true;
    },
    exitLoadingState(state: AppState) {
      state.isLoading = false;
    },
    updateProgress(
      state: AppState,
      { payload: update }: PayloadAction<ProgressUpdate>
    ) {
      if (
        seqFromProgressUpdate(update) >
        seqFromProgressUpdate(state.progressUpdate)
      ) {
        state.progressUpdate = update;
      }
    },
    updateProgressFinished(state: AppState) {
      state.progressUpdate = null;
    },
  },
});
export default appSlice.reducer;
export const {
  bootstrapHasLoaded,
  navigateTo,
  enterLoadingState,
  exitLoadingState,
  updateProgress,
  updateProgressFinished,
} = appSlice.actions;

////////////////////////////////////////

export const $isBootstrapped = ({ [APP_SLICE]: app }: MainRootState) =>
  app.bootstrapped;

export const $currentMode = ({ [APP_SLICE]: app }: { [APP_SLICE]: AppState }) =>
  app.currentMode;

export const $isLoading = ({ [APP_SLICE]: app }: { [APP_SLICE]: AppState }) =>
  app.isLoading || !app.bootstrapped;

export const $progress = ({ [APP_SLICE]: app }: { [APP_SLICE]: AppState }) =>
  app.progressUpdate;
