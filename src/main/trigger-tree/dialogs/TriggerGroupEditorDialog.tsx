import React, { useState } from 'react';
import { useDispatch } from 'react-redux';

import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import DialogTitle from '@mui/material/DialogTitle';
import TextField from '@mui/material/TextField';

import { applyDeltas } from '../../../features/triggers/triggersSlice';
import { TriggerGroup } from '../../../generated/TriggerGroup';
import { saveTriggerGroup } from '../../../ipc';

const TriggerGroupEditorDialog: React.FC<{
  triggerGroup: TriggerGroup;
  close: () => void;
}> = ({ triggerGroup, close }) => {
  const dispatch = useDispatch();

  const [nameInput, setNameInput] = useState(triggerGroup.name);
  const [commentInput, setCommentInput] = useState(triggerGroup.comment);

  const save = async () => {
    const name = nameInput.trim();
    const comment = commentInput?.trim();

    const deltas = await saveTriggerGroup(
      triggerGroup.id,
      name,
      comment ? comment : null
    );
    dispatch(applyDeltas(deltas));

    close();
  };

  const nameError = nameInput.trim() ? undefined : 'Name cannot be blank';

  return (
    <Dialog open={true}>
      <DialogTitle>Editing Trigger Group "{triggerGroup.name}"</DialogTitle>
      <DialogContent>
        <TextField
          label="Trigger Group Name"
          variant="outlined"
          error={!!nameError}
          helperText={nameError}
          defaultValue={nameInput}
          onChange={(e) => setNameInput(e.target.value)}
          sx={{ mt: 1, mb: 1 }}
          fullWidth
        />
        <TextField
          label="Comment (Optional)"
          variant="outlined"
          defaultValue={commentInput}
          onChange={(e) => setCommentInput(e.target.value)}
          sx={{ mt: 1, mb: 1 }}
          fullWidth
          multiline
        />
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={close}>
          Cancel
        </Button>
        <Button variant="contained" onClick={save} disabled={!!nameError}>
          Save
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default TriggerGroupEditorDialog;
