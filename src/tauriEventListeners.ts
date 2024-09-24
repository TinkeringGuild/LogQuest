import { Action, Dispatch } from '@reduxjs/toolkit';
import { listen } from '@tauri-apps/api/event';
import { uniqueId } from 'lodash';

import {
  appendMessage,
  initOverlay,
  removeMessage,
  setEditable,
} from './features/overlay/overlaySlice';
import { initTimers, timerStateUpdate } from './features/timers/timersSlice';
import {
  CROSS_DISPATCH_EVENT_NAME,
  OVERLAY_EDITABLE_CHANGED_EVENT_NAME,
  OVERLAY_MESSAGE_EVENT_NAME,
  OVERLAY_STATE_UPDATE_EVENT_NAME,
} from './generated/constants';
import { TimerStateUpdate } from './generated/TimerStateUpdate';
import { getOverlayBootstrap, startTimersSync } from './ipc';
import { OverlayDispatch } from './OverlayStore';

// TODO: This should come from an overlay config setting
const OVERLAY_MESSAGE_LIFETIME_MILLIS = 14 * 1000;

export const initCrossDispatchListener = (dispatch: Dispatch) => {
  return listen<Action>(CROSS_DISPATCH_EVENT_NAME, ({ payload: action }) => {
    return dispatch(action);
  });
};

export const bootstrapOverlay = (dispatch: Dispatch) => {
  getOverlayBootstrap().then((overlayBootstrap) =>
    dispatch(initOverlay(overlayBootstrap))
  );
};

export const initOverlayStateListeners = (dispatch: Dispatch) => {
  initTimersSync(dispatch);
  initOverlayTimersListener(dispatch);
  initOverlayMessageListener(dispatch);
  initOverlayEditableListener(dispatch);
};

const initTimersSync = (dispatch: Dispatch) => {
  startTimersSync().then((timerLifetimes) => {
    dispatch(initTimers(timerLifetimes));
  });
};

const initOverlayTimersListener = (dispatch: Dispatch) => {
  return listen<TimerStateUpdate>(
    OVERLAY_STATE_UPDATE_EVENT_NAME,
    ({ payload: update }) => {
      dispatch(timerStateUpdate(update));
    }
  );
};

const initOverlayMessageListener = (dispatch: OverlayDispatch) => {
  return listen<string>(OVERLAY_MESSAGE_EVENT_NAME, ({ payload: text }) => {
    const id = uniqueId('overlay-message-');
    // console.log('Overlay Message Event: ', text);
    dispatch(appendMessage({ id, text }));
    removeMessageLater(id, OVERLAY_MESSAGE_LIFETIME_MILLIS, dispatch);
  });
};

const initOverlayEditableListener = (dispatch: Dispatch) => {
  return listen<boolean>(
    OVERLAY_EDITABLE_CHANGED_EVENT_NAME,
    ({ payload: newValue }) => {
      dispatch(setEditable(newValue));
    }
  );
};

function removeMessageLater(
  id: string,
  delayMillis: number,
  dispatch: OverlayDispatch
) {
  return setTimeout(() => {
    dispatch(removeMessage(id));
  }, delayMillis);
}
