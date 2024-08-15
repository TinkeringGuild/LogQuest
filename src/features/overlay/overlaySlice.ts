import { createSlice } from '@reduxjs/toolkit';

import { OverlayState } from '../../generated/OverlayState';
import { MainRootState } from '../../MainStore';

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

export const selectOverlayEditable = (state: MainRootState) =>
  state.overlay.overlay_editable;
