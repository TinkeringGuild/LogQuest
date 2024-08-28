import { configureStore } from '@reduxjs/toolkit';

import timersSlice, { TIMERS_SLICE } from './features/timers/timersSlice';
import { bootstrapOverlay, initOverlayListeners } from './tauriEvents';
import overlayReducer, { OVERLAY_SLICE } from './features/overlay/overlaySlice';

const store = configureStore({
  reducer: {
    [TIMERS_SLICE]: timersSlice,
    [OVERLAY_SLICE]: overlayReducer,
  },
});

bootstrapOverlay(store.dispatch);
initOverlayListeners(store.dispatch);

export default store;

export type OverlayRootState = ReturnType<typeof store.getState>;
export type OverlayDispatch = typeof store.dispatch;
