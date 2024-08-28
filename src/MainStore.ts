import { configureStore } from '@reduxjs/toolkit';

import appReducer, { APP_SLICE } from './features/app/appSlice';
import configReducer, { CONFIG_SLICE } from './features/config/configSlice';
import overlayReducer, { OVERLAY_SLICE } from './features/overlay/overlaySlice';
import triggersReducer, {
  TRIGGERS_SLICE,
} from './features/triggers/triggersSlice';
import { initOverlayListeners } from './tauriEvents';

const store = configureStore({
  reducer: {
    [APP_SLICE]: appReducer,
    [CONFIG_SLICE]: configReducer,
    [TRIGGERS_SLICE]: triggersReducer,
    [OVERLAY_SLICE]: overlayReducer,
  },
});

initOverlayListeners(store.dispatch);

export type MainRootState = ReturnType<typeof store.getState>;
export type MainDispatch = typeof store.dispatch;

export default store;
