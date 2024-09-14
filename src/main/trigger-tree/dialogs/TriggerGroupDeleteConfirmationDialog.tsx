import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';
import DialogTitle from '@mui/material/DialogTitle';

import { TriggerGroup } from '../../../generated/TriggerGroup';

const TriggerGroupDeleteConfirmationDialog: React.FC<{
  triggerGroup: TriggerGroup;
  onDelete: () => void;
  close: () => void;
}> = ({ triggerGroup, onDelete, close }) => (
  <Dialog open={true}>
    <DialogTitle>Confirm deleting "{triggerGroup.name}"</DialogTitle>
    <DialogContent>
      <DialogContentText>
        Deleting this "<strong>{triggerGroup.name}</strong>" Trigger Group will
        also delete <em>ALL nested Triggers and Trigger Groups within it.</em>
      </DialogContentText>
      <DialogContentText sx={{ mt: 1 }}>
        Are you sure you wish to continue?
      </DialogContentText>
    </DialogContent>
    <DialogActions>
      <Button variant="outlined" onClick={close}>
        Cancel
      </Button>
      <Button color="error" variant="contained" onClick={onDelete}>
        Delete Trigger Group and everything in it
      </Button>
    </DialogActions>
  </Dialog>
);

export default TriggerGroupDeleteConfirmationDialog;
