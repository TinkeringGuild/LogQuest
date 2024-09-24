import React, { useState } from 'react';

import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import DialogTitle from '@mui/material/DialogTitle';
import TextField from '@mui/material/TextField';

const TriggerGroupEditorDialog: React.FC<{
  name: string;
  comment: string | null;
  onSave: (name: string, comment: string | null) => void;
  close: () => void;
}> = ({ name, comment, onSave, close }) => {
  const [nameInput, setNameInput] = useState(name);
  const [commentInput, setCommentInput] = useState(comment);

  const save = async () => {
    const name = nameInput.trim();
    const comment = commentInput?.trim() || null;
    onSave(name, comment);
    close();
  };

  const nameError = nameInput.trim() ? undefined : 'Name cannot be blank';

  return (
    <Dialog open={true}>
      <DialogTitle>Editing Trigger Group "{name}"</DialogTitle>
      <DialogContent>
        <TextField
          label="Trigger Group Name"
          variant="outlined"
          error={!!nameError}
          helperText={nameError || ' '}
          defaultValue={nameInput}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !nameError) {
              save();
            }
          }}
          onChange={(e) => setNameInput(e.target.value)}
          sx={{ mt: 1, mb: 0 }}
          fullWidth
          autoFocus
        />
        <TextField
          label="Comment (Optional)"
          variant="outlined"
          defaultValue={commentInput || ''}
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
