import { createSlice } from '@reduxjs/toolkit';
import { remove } from 'lodash';

import { OverlayState } from '../../generated/OverlayState';

export const OVERLAY_SLICE = 'overlay';

export type OverlayMessage = { id: string; text: string };

interface OverlaySliceState {
  backend: OverlayState;
  messages: OverlayMessage[];
}

const INITIAL_OVERLAY_STATE: OverlaySliceState = {
  backend: {
    overlay_editable: false,
    overlay_mode: 'Default', // I don't like setting this default here
  },
  messages: [],
};

const overlaySlice = createSlice({
  name: OVERLAY_SLICE,
  initialState: INITIAL_OVERLAY_STATE,
  reducers: {
    initOverlay: (state, { payload }) => {
      state.backend = payload;
    },
    setEditable: (state, { payload: newValue }: { payload: boolean }) => {
      state.backend.overlay_editable = newValue;
    },
    appendMessage: (
      state,
      { payload: message }: { payload: OverlayMessage }
    ) => {
      state.messages.push(message);
    },
    removeMessage: (state, { payload: messageID }: { payload: string }) => {
      remove(state.messages, ({ id }) => id === messageID);
    },
  },
});
export default overlaySlice.reducer;
export const { initOverlay, setEditable, appendMessage, removeMessage } =
  overlaySlice.actions;

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
