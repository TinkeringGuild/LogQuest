import React, { useState } from 'react';

import DeleteForeverOutlined from '@mui/icons-material/DeleteForeverOutlined';
import Edit from '@mui/icons-material/Edit';
import VerticalAlignBottom from '@mui/icons-material/VerticalAlignBottom';
import VerticalAlignTop from '@mui/icons-material/VerticalAlignTop';
import Divider from '@mui/material/Divider';
import ListItemIcon from '@mui/material/ListItemIcon';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import { TriggerGroup } from '../../../generated/TriggerGroup';
import TriggerGroupDeleteConfirmationDialog from '../dialogs/TriggerGroupDeleteConfirmationDialog';

const TriggerGroupContextMenu: React.FC<{
  triggerGroup: TriggerGroup;
  top: number;
  left: number;
  onEdit: () => void;
  onDelete: () => void;
  onInsertTrigger: (offset: 0 | 1) => void;
  onInsertGroup: (offset: 0 | 1) => void;
  close: () => void;
}> = ({
  triggerGroup,
  top,
  left,
  onEdit,
  onDelete,
  onInsertTrigger,
  onInsertGroup,
  close,
}) => {
  const [deleteConfirmationIsOpen, setDeleteConfirmationIsOpen] =
    useState(false);

  if (deleteConfirmationIsOpen) {
    return (
      <TriggerGroupDeleteConfirmationDialog
        triggerGroup={triggerGroup}
        onDelete={onDelete}
        close={() => {
          setDeleteConfirmationIsOpen(false);
          close();
        }}
      />
    );
  }

  return (
    <Menu
      open={true}
      onClose={close}
      anchorReference="anchorPosition"
      anchorPosition={{ top, left }}
    >
      <MenuItem
        onClick={() => {
          onEdit();
          close();
        }}
      >
        <ListItemIcon>
          <Edit />
        </ListItemIcon>
        Edit Trigger Group
      </MenuItem>

      <MenuItem onClick={() => setDeleteConfirmationIsOpen(true)}>
        <ListItemIcon>
          <DeleteForeverOutlined />
        </ListItemIcon>
        Delete Trigger Group
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
};

export default TriggerGroupContextMenu;
