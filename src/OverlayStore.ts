import { configureStore } from '@reduxjs/toolkit';

import overlayReducer, { OVERLAY_SLICE } from './features/overlay/overlaySlice';
import timersSlice, { TIMERS_SLICE } from './features/timers/timersSlice';
import {
  bootstrapOverlay,
  initCrossDispatchListener,
  initOverlayStateListeners,
} from './tauriEventListeners';

const store = configureStore({
  reducer: {
    [TIMERS_SLICE]: timersSlice,
    [OVERLAY_SLICE]: overlayReducer,
  },
});

bootstrapOverlay(store.dispatch);
initOverlayStateListeners(store.dispatch);
initCrossDispatchListener(store.dispatch);

export default store;

export type OverlayRootState = ReturnType<typeof store.getState>;
export type OverlayDispatch = typeof store.dispatch;
