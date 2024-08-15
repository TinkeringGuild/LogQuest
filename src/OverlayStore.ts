import { configureStore } from '@reduxjs/toolkit';
import timersSlice from './features/timers/timersSlice';

const store = configureStore({
  reducer: {
    timers: timersSlice,
  },
});

export default store;

export type OverlayRootState = ReturnType<typeof store.getState>;
export type OverlayDispatch = typeof store.dispatch;
