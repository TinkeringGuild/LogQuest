import { Action, Middleware } from '@reduxjs/toolkit';

import { OVERLAY_SLICE } from './features/overlay/overlaySlice';
import { dispatchToOverlay } from './ipc';

const SET_OVERLAY_OPACITY_ACTION = `${OVERLAY_SLICE}/setOverlayOpacity`;

const crossDispatchMiddleware: Middleware =
  (_store) => (next) => (unknownAction) => {
    const action: Action = unknownAction as Action;
    switch (action.type) {
      case SET_OVERLAY_OPACITY_ACTION:
        dispatchToOverlay(action);
    }

    return next(action);
  };

export default crossDispatchMiddleware;
