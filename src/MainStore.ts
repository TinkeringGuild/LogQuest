import { configureStore } from '@reduxjs/toolkit';

import crossDispatchMiddleware from './crossDispatchMiddleware';
import appReducer, { APP_SLICE } from './features/app/appSlice';
import configReducer, { CONFIG_SLICE } from './features/config/configSlice';
import overlayReducer, { OVERLAY_SLICE } from './features/overlay/overlaySlice';
import triggerEditorReducer, {
  TRIGGER_EDITOR_SLICE,
} from './features/triggers/triggerEditorSlice';
import triggersReducer, {
  TRIGGERS_SLICE,
} from './features/triggers/triggersSlice';
import { initOverlayStateListeners } from './tauriEventListeners';

const store = configureStore({
  reducer: {
    [APP_SLICE]: appReducer,
    [CONFIG_SLICE]: configReducer,
    [TRIGGERS_SLICE]: triggersReducer,
    [TRIGGER_EDITOR_SLICE]: triggerEditorReducer,
    [OVERLAY_SLICE]: overlayReducer,
  },
  middleware: (getDefaultMiddleware) =>
    // serializableCheck must be disabled because I pass reducer functions into editorReducer actions
    // to greatly simplify the logic of mutating deeply nested Trigger values. In the future, it might
    // be possible to insert custom middleware that automatically applies the reducer and passes through
    // a new Immer draft object that can be updated, however that would only be necessary if I wanted to
    // use the time-travel debugging features of Redux.
    getDefaultMiddleware({ serializableCheck: false }).concat(
      crossDispatchMiddleware
    ),
});

initOverlayStateListeners(store.dispatch);

export type MainRootState = ReturnType<typeof store.getState>;
export type MainDispatch = typeof store.dispatch;
export type MainSelector<T> = (state: MainRootState) => T;

export default store;
