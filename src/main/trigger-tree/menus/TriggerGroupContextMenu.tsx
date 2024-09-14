import React, { useState } from 'react';

import DeleteForeverOutlined from '@mui/icons-material/DeleteForeverOutlined';
import Edit from '@mui/icons-material/Edit';
import ListItemIcon from '@mui/material/ListItemIcon';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import TriggerGroupDeleteConfirmationDialog from '../dialogs/TriggerGroupDeleteConfirmationDialog';
import { TriggerGroup } from '../../../generated/TriggerGroup';

const TriggerGroupContextMenu: React.FC<{
  triggerGroup: TriggerGroup;
  top: number;
  left: number;
  onEdit: () => void;
  onDelete: () => void;
  close: () => void;
}> = ({ triggerGroup, top, left, onEdit, onDelete, close }) => {
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
    </Menu>
  );
};

export default TriggerGroupContextMenu;
