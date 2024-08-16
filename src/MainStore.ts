import { configureStore } from '@reduxjs/toolkit';

import appReducer, { APP_SLICE } from './features/app/appSlice';
import configReducer, { CONFIG_SLICE } from './features/config/configSlice';
import triggersReducer, {
  TRIGGERS_SLICE,
} from './features/triggers/triggersSlice';
import overlayReducer, { OVERLAY_SLICE } from './features/overlay/overlaySlice';

const store = configureStore({
  reducer: {
    [APP_SLICE]: appReducer,
    [CONFIG_SLICE]: configReducer,
    [TRIGGERS_SLICE]: triggersReducer,
    [OVERLAY_SLICE]: overlayReducer,
  },
});

export type MainRootState = ReturnType<typeof store.getState>;
export type MainDispatch = typeof store.dispatch;

export default store;
