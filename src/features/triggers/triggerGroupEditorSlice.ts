import { createSlice } from '@reduxjs/toolkit';
import { TriggerGroup } from '../../generated/TriggerGroup';

export const TRIGGER_GROUP_EDITOR_SLICE = 'trigger-group-editor';

export type TriggerGroupEditorState = {
  draft: TriggerGroup | null;
};

const INITIAL_TRIGGER_GROUP_EDITOR_STATE = {
  draft: null,
} satisfies TriggerGroupEditorState;

const triggerGroupEditorSlice = createSlice({
  name: TRIGGER_GROUP_EDITOR_SLICE,
  initialState: INITIAL_TRIGGER_GROUP_EDITOR_STATE,
  reducers: {
    cancelEditing(slice: TriggerGroupEditorState) {
      slice.draft = null;
    },
  },
});

export default triggerGroupEditorSlice.reducer;

export const { cancelEditing } = triggerGroupEditorSlice.actions;
