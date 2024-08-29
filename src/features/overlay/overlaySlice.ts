import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { remove } from 'lodash';

import { OverlayState } from '../../generated/OverlayState';
import { DEFAULT_OVERLAY_OPACITY } from '../../generated/constants';

export const OVERLAY_SLICE = 'overlay';

export type OverlayMessage = { id: string; text: string };

interface OverlaySliceState {
  backend: OverlayState;
  messages: OverlayMessage[];
}

const INITIAL_OVERLAY_STATE: OverlaySliceState = {
  backend: {
    overlay_editable: false,
    overlay_mode: null,
    overlay_opacity: DEFAULT_OVERLAY_OPACITY,
  },
  messages: [],
};

const overlaySlice = createSlice({
  name: OVERLAY_SLICE,
  initialState: INITIAL_OVERLAY_STATE,
  reducers: {
    initOverlay: (state, { payload }: PayloadAction<OverlayState>) => {
      state.backend = payload;
    },
    setEditable: (state, { payload: newValue }: PayloadAction<boolean>) => {
      state.backend.overlay_editable = newValue;
    },
    appendMessage: (
      state,
      { payload: message }: PayloadAction<OverlayMessage>
    ) => {
      state.messages.push(message);
    },
    removeMessage: (state, { payload: messageID }: PayloadAction<string>) => {
      remove(state.messages, ({ id }) => id === messageID);
    },
    setOverlayOpacity: (state, { payload: opacity }: PayloadAction<number>) => {
      state.backend.overlay_opacity = opacity;
    },
  },
});
export default overlaySlice.reducer;
export const {
  initOverlay,
  setEditable,
  appendMessage,
  removeMessage,
  setOverlayOpacity,
} = overlaySlice.actions;

export const $overlayEditable = ({
  [OVERLAY_SLICE]: overlay,
}: {
  [OVERLAY_SLICE]: OverlaySliceState;
}) => overlay.backend.overlay_editable;

export const $overlayMessages = ({
  [OVERLAY_SLICE]: overlay,
}: {
  [OVERLAY_SLICE]: OverlaySliceState;
}) => overlay.messages;

export const $overlayOpacity = ({
  [OVERLAY_SLICE]: overlay,
}: {
  [OVERLAY_SLICE]: OverlaySliceState;
}) => overlay.backend.overlay_opacity;
