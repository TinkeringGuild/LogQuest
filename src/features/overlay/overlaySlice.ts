import { createSlice } from '@reduxjs/toolkit';

import { OverlayState } from '../../generated/OverlayState';
import { RootState } from '../../store';

const INITIAL_OVERLAY_STATE: OverlayState = {
  overlay_editable: false,
};

const overlaySlice = createSlice({
  name: 'overlay',
  initialState: INITIAL_OVERLAY_STATE,
  reducers: {
    initOverlay: (_state, { payload }) => payload,
  },
});

export default overlaySlice.reducer;

export const { initOverlay } = overlaySlice.actions;

export const selectOverlayEditable = (state: RootState) =>
  state.overlay.overlay_editable;
