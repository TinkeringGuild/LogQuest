import ControlPointDuplicateOutlined from '@mui/icons-material/ControlPointDuplicateOutlined';
import DeleteForeverOutlined from '@mui/icons-material/DeleteForeverOutlined';
import VerticalAlignBottom from '@mui/icons-material/VerticalAlignBottom';
import VerticalAlignTop from '@mui/icons-material/VerticalAlignTop';
import Divider from '@mui/material/Divider';
import ListItemIcon from '@mui/material/ListItemIcon';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';

export const TriggerContextMenu: React.FC<{
  top: number;
  left: number;
  onDuplicate: () => void;
  onDelete: () => void;
  onInsertTrigger: (offset: 0 | 1) => void;
  onInsertGroup: (offset: 0 | 1) => void;
  close: () => void;
}> = ({
  top,
  left,
  onDuplicate,
  onDelete,
  onInsertTrigger,
  onInsertGroup,
  close,
}) => (
  <Menu
    open={true}
    onClose={close}
    anchorReference="anchorPosition"
    anchorPosition={{ top, left }}
  >
    <MenuItem
      onClick={() => {
        onDelete();
        close();
      }}
    >
      <ListItemIcon>
        <DeleteForeverOutlined />
      </ListItemIcon>
      Delete Trigger
    </MenuItem>

    <MenuItem
      onClick={() => {
        onDuplicate();
        close();
      }}
    >
      <ListItemIcon>
        <ControlPointDuplicateOutlined />
      </ListItemIcon>
      Duplicate Trigger
    </MenuItem>

    <Divider />

    <MenuItem
      onClick={() => {
        onInsertTrigger(0);
        close();
      }}
    >
      <ListItemIcon>
        <VerticalAlignTop />
      </ListItemIcon>
      Create new Trigger above
    </MenuItem>

    <MenuItem
      onClick={() => {
        onInsertTrigger(1);
        close();
      }}
    >
      <ListItemIcon>
        <VerticalAlignBottom />
      </ListItemIcon>
      Create new Trigger below
    </MenuItem>

    <Divider />

    <MenuItem
      onClick={() => {
        onInsertGroup(0);
        close();
      }}
    >
      <ListItemIcon>
        <VerticalAlignTop />
      </ListItemIcon>
      Create new Trigger Group above
    </MenuItem>

    <MenuItem
      onClick={() => {
        onInsertGroup(1);
        close();
      }}
    >
      <ListItemIcon>
        <VerticalAlignBottom />
      </ListItemIcon>
      Create new Trigger Group below
    </MenuItem>
  </Menu>
);

export default TriggerContextMenu;
