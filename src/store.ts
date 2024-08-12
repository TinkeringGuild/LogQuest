import { configureStore } from '@reduxjs/toolkit';

import appReducer from './features/app/appSlice';
import configReducer from './features/config/configSlice';
import triggersReducer from './features/triggers/triggersSlice';
import overlayReducer from './features/overlay/overlaySlice';

const store = configureStore({
  reducer: {
    app: appReducer,
    config: configReducer,
    triggers: triggersReducer,
    overlay: overlayReducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

export default store;
