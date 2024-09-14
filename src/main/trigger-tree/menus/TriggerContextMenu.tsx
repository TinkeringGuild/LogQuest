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
  onDelete: () => void;
  close: () => void;
}> = ({ top, left, onDelete, close }) => (
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
    <MenuItem>
      <ListItemIcon>
        <ControlPointDuplicateOutlined />
      </ListItemIcon>
      Duplicate Trigger
    </MenuItem>
    <Divider />
    <MenuItem>
      <ListItemIcon>
        <VerticalAlignTop />
      </ListItemIcon>
      Create new Trigger above
    </MenuItem>
    <MenuItem>
      <ListItemIcon>
        <VerticalAlignBottom />
      </ListItemIcon>
      Create new Trigger below
    </MenuItem>
    <Divider />
    <MenuItem>
      <ListItemIcon>
        <VerticalAlignTop />
      </ListItemIcon>
      Create new Trigger Group above
    </MenuItem>
    <MenuItem>
      <ListItemIcon>
        <VerticalAlignBottom />
      </ListItemIcon>
      Create new Trigger Group below
    </MenuItem>
  </Menu>
);

export default TriggerContextMenu;
