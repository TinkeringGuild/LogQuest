import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { MainRootState } from '../../MainStore';
import { Character } from '../../generated/Character';
import { ProgressUpdate } from '../../generated/ProgressUpdate';
import { ReactorState } from '../../generated/ReactorState';
import { UUID } from '../../generated/UUID';
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
  reactor: ReactorState | null;
}

const INITIAL_APP_STATE: AppState = {
  bootstrapped: false,
  currentMode: 'overview',
  isLoading: false,
  progressUpdate: null,
  reactor: null,
};

export type AppSelector<T> = (slice: AppState) => T;

const appSlice = createSlice({
  name: APP_SLICE,
  initialState: INITIAL_APP_STATE,
  reducers: {
    initReactor(
      slice: AppState,
      { payload: reactorState }: PayloadAction<ReactorState>
    ) {
      slice.reactor = reactorState;
    },

    updateActivedTriggerTagIDs(
      slice: AppState,
      { payload }: PayloadAction<UUID[]>
    ) {
      if (!slice.reactor) {
        return;
      }
      slice.reactor.active_trigger_tags = payload;
    },

    bootstrapHasLoaded(slice: AppState) {
      slice.bootstrapped = true;
    },

    setCurrentCharacter(
      slice: AppState,
      { payload: characterMaybe }: PayloadAction<Character | null>
    ) {
      if (slice.reactor) {
        slice.reactor.current_character = characterMaybe;
      }
    },

    navigateTo(slice: AppState, { payload: mode }: PayloadAction<MODE>) {
      slice.currentMode = mode;
    },

    enterLoadingState(slice: AppState) {
      slice.isLoading = true;
    },

    exitLoadingState(slice: AppState) {
      slice.isLoading = false;
    },

    updateProgress(
      slice: AppState,
      { payload: update }: PayloadAction<ProgressUpdate>
    ) {
      if (
        seqFromProgressUpdate(update) >
        seqFromProgressUpdate(slice.progressUpdate)
      ) {
        slice.progressUpdate = update;
      }
    },

    updateProgressFinished(slice: AppState) {
      slice.progressUpdate = null;
    },
  },
});

export default appSlice.reducer;

export const {
  bootstrapHasLoaded,
  enterLoadingState,
  exitLoadingState,
  initReactor,
  navigateTo,
  setCurrentCharacter,
  updateActivedTriggerTagIDs,
  updateProgress,
  updateProgressFinished,
} = appSlice.actions;

////////////////////////////////////////

export function appSelector<T>(
  selector: AppSelector<T>
): (state: MainRootState) => T {
  return (state: MainRootState) => selector(state[APP_SLICE]);
}
export const $isBootstrapped = appSelector(({ bootstrapped }) => bootstrapped);

export const $currentMode = appSelector(({ currentMode }) => currentMode);

export const $isLoading = appSelector(
  ({ isLoading, bootstrapped }) => isLoading || !bootstrapped
);

export const $progress = appSelector(({ progressUpdate }) => progressUpdate);

export const $currentCharacter = appSelector(
  ({ reactor }) => reactor?.current_character
);

export const $activeTriggerTags = appSelector(
  ({ reactor }) => reactor?.active_trigger_tags
);
