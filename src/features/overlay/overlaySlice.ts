import { createSlice } from '@reduxjs/toolkit';

import { OverlayState } from '../../generated/OverlayState';

export const OVERLAY_SLICE = 'overlay';

const INITIAL_OVERLAY_STATE: OverlayState = {
  overlay_editable: false,
};

const overlaySlice = createSlice({
  name: OVERLAY_SLICE,
  initialState: INITIAL_OVERLAY_STATE,
  reducers: {
    initOverlay: (_state, { payload }) => payload,
  },
});
export default overlaySlice.reducer;
export const { initOverlay } = overlaySlice.actions;

export const $overlayEditable = ({
  [OVERLAY_SLICE]: overlay,
}: {
  [OVERLAY_SLICE]: OverlayState;
}) => overlay.overlay_editable;
