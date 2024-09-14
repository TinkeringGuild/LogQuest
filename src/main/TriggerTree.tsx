import { cloneDeep } from 'lodash';
import React, { createContext, useContext, useRef, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import {
  ControlPointDuplicateOutlined,
  Edit,
  VerticalAlignBottom,
  VerticalAlignTop,
} from '@mui/icons-material';
import DeleteForeverOutlined from '@mui/icons-material/DeleteForeverOutlined';
import DownloadingIcon from '@mui/icons-material/Downloading';
import {
  Box,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Divider,
  ListItemIcon,
  Menu,
  MenuItem,
  Switch,
  TextField,
} from '@mui/material';
import Button from '@mui/material/Button';

import openGINATriggerFileDialog from '../dialogs/importGINAFile';
import {
  $draftTrigger,
  editTriggerDraft,
} from '../features/triggers/triggerEditorSlice';
import {
  $activeTriggerTag,
  $activeTriggerTagID,
  $topLevel,
  $trigger,
  $triggerGroup,
  $triggerGroupMaybe,
  $triggerTagsHavingTrigger,
  activateTriggerTagID,
  applyDeltas,
} from '../features/triggers/triggersSlice';
import { TriggerGroupDescendant } from '../generated/TriggerGroupDescendant';
import { UUID } from '../generated/UUID';
import {
  addTriggerToTag,
  createTriggerTag,
  deleteTrigger,
  deleteTriggerGroup,
  removeTriggerFromTag,
  saveTriggerGroup,
} from '../ipc';
import store from '../MainStore';
import TriggerTagChanger from './TriggerTagChanger';

import './TriggerTree.css';

type TriggerContextMenuState = {
  triggerID: string;
  mouseX: number;
  mouseY: number;
} | null;

type TriggerGroupContextMenuState = {
  groupID: string;
  mouseX: number;
  mouseY: number;
} | null;

const TriggerMenuContext = createContext<
  | null
  | [UUID | null, React.Dispatch<React.SetStateAction<TriggerContextMenuState>>]
>(null);

const TriggerGroupMenuContext = createContext<
  | null
  | [
      UUID | null,
      React.Dispatch<React.SetStateAction<TriggerGroupContextMenuState>>,
    ]
>(null);

const TriggerTree: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const top: TriggerGroupDescendant[] = useSelector($topLevel);
  const activeTriggerTag = useSelector($activeTriggerTag);

  const activeTriggersSet: Set<string> | null =
    activeTriggerTag && new Set(activeTriggerTag.triggers);

  const [triggerContextMenu, setTriggerContextMenu] =
    useState<TriggerContextMenuState>(null);

  const [triggerGroupContextMenu, setTriggerGroupContextMenu] =
    useState<TriggerGroupContextMenuState>(null);

  const closeTriggerContextMenu = () => setTriggerContextMenu(null);

  return (
    <div className="trigger-tree trigger-browser-scrollable-container">
      <div className="trigger-browser-scrollable-content scrollable-content central-content">
        <Box justifyItems="right">
          <div style={{ textAlign: 'right' }}>
            <Button
              size="small"
              variant="contained"
              startIcon={<DownloadingIcon />}
              onClick={() => openGINATriggerFileDialog(dispatch)}
            >
              Import GINA Export
            </Button>
          </div>
        </Box>
        <TriggerTagChanger
          onChange={(tagIDMaybe) => dispatch(activateTriggerTagID(tagIDMaybe))}
          onCreate={async (name) => {
            const deltas = await createTriggerTag(name);
            dispatch(applyDeltas(deltas));
            const creation = deltas.find(
              (delta) => delta.variant === 'TriggerTagCreated'
            );
            if (creation) {
              dispatch(activateTriggerTagID(creation.value.id));
            }
          }}
        />
        <TriggerMenuContext.Provider
          value={[triggerContextMenu?.triggerID || null, setTriggerContextMenu]}
        >
          <TriggerGroupMenuContext.Provider
            value={[
              triggerGroupContextMenu?.groupID || null,
              setTriggerGroupContextMenu,
            ]}
          >
            <div>
              {top.length ? (
                <ul>
                  {top.map((tgd) =>
                    tgd.variant === 'T' ? (
                      <TriggerListItem
                        key={tgd.value}
                        triggerID={tgd.value}
                        activeTriggers={activeTriggersSet}
                      />
                    ) : (
                      <TriggerGroupListItem
                        key={tgd.value}
                        groupID={tgd.value}
                        activeTriggers={activeTriggersSet}
                      />
                    )
                  )}
                </ul>
              ) : (
                <p>You have not created any triggers yet.</p>
              )}
            </div>
          </TriggerGroupMenuContext.Provider>
        </TriggerMenuContext.Provider>
      </div>
      <Menu
        open={triggerContextMenu !== null}
        onClose={closeTriggerContextMenu}
        anchorReference="anchorPosition"
        anchorPosition={
          triggerContextMenu !== null
            ? {
                top: triggerContextMenu.mouseY,
                left: triggerContextMenu.mouseX,
              }
            : undefined
        }
      >
        <MenuItem
          onClick={async () => {
            if (!triggerContextMenu) {
              return;
            }
            const deltas = await deleteTrigger(triggerContextMenu.triggerID);
            dispatch(applyDeltas(deltas));
            setTriggerContextMenu(null);
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
      <TriggerGroupContextMenu
        menuState={triggerGroupContextMenu}
        setMenuState={setTriggerGroupContextMenu}
      />
    </div>
  );
};

const TriggerGroupContextMenu: React.FC<{
  menuState: TriggerGroupContextMenuState | null;
  setMenuState: React.Dispatch<
    React.SetStateAction<TriggerGroupContextMenuState>
  >;
}> = ({ menuState, setMenuState }) => {
  const dispatch = useDispatch();

  const [deleteConfirmationGroupID, setDeleteConfirmationGroupID] =
    React.useState<UUID | null>(null);

  const [triggerGroupEditorGroupID, setTriggerGroupEditorGroupID] =
    React.useState<UUID | null>(null);

  const triggerGroup = useSelector(
    $triggerGroupMaybe(deleteConfirmationGroupID)
  );

  const closeMenu = () => setMenuState(null);
  return (
    <>
      <Menu
        open={menuState !== null}
        onClose={closeMenu}
        anchorReference="anchorPosition"
        anchorPosition={
          menuState !== null
            ? {
                top: menuState.mouseY,
                left: menuState.mouseX,
              }
            : undefined
        }
      >
        <MenuItem
          onClick={() => {
            setTriggerGroupEditorGroupID(menuState!.groupID);
            closeMenu();
          }}
        >
          <ListItemIcon>
            <Edit />
          </ListItemIcon>
          Edit Trigger Group
        </MenuItem>
        <MenuItem
          onClick={() => {
            setDeleteConfirmationGroupID(menuState!.groupID);
            closeMenu();
          }}
        >
          <ListItemIcon>
            <DeleteForeverOutlined />
          </ListItemIcon>
          Delete Trigger Group
        </MenuItem>
      </Menu>
      <Dialog open={deleteConfirmationGroupID !== null}>
        <DialogTitle>
          Confirm deleting "{triggerGroup && triggerGroup.name}"
        </DialogTitle>
        <DialogContent>
          <DialogContentText>
            {triggerGroup && (
              <>
                Deleting this "<strong>{triggerGroup.name}</strong>" Trigger
                Group will also delete{' '}
                <em>ALL nested Triggers and Trigger Groups within it.</em>
              </>
            )}
          </DialogContentText>
          <DialogContentText sx={{ mt: 1 }}>
            Are you sure you wish to continue?
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button
            variant="outlined"
            onClick={() => {
              setDeleteConfirmationGroupID(null);
            }}
          >
            Cancel
          </Button>
          <Button
            color="error"
            variant="contained"
            onClick={async () => {
              const deltas = await deleteTriggerGroup(
                deleteConfirmationGroupID!
              );
              dispatch(applyDeltas(deltas));
              setDeleteConfirmationGroupID(null);
            }}
          >
            Delete Trigger Group and everything in it
          </Button>
        </DialogActions>
      </Dialog>
      {!!triggerGroupEditorGroupID && (
        <TriggerGroupEditorDialog
          triggerGroupID={triggerGroupEditorGroupID}
          onSave={async ({ name, comment }) => {
            const deltas = await saveTriggerGroup(
              triggerGroupEditorGroupID,
              name,
              comment
            );
            dispatch(applyDeltas(deltas));
            setTriggerGroupEditorGroupID(null);
          }}
          onCancel={() => setTriggerGroupEditorGroupID(null)}
        />
      )}
    </>
  );
};

const TriggerGroupEditorDialog: React.FC<{
  triggerGroupID: UUID;
  onSave: (group: { name: string; comment: string | null }) => void;
  onCancel: () => void;
}> = ({ triggerGroupID, onSave, onCancel }) => {
  const nameRef = useRef<HTMLInputElement | null>(null);
  const commentRef = useRef<HTMLInputElement | null>(null);

  const triggerGroup = useSelector($triggerGroup(triggerGroupID));

  const saveClicked = () => {
    if (nameRef.current && commentRef.current) {
      const name = nameRef.current.value.trim();
      let comment: string | null = commentRef.current.value.trim();

      onSave({ name, comment: comment ? comment : null });
    }
  };

  return (
    <Dialog open={true}>
      <DialogTitle>Editing Trigger Group "{triggerGroup.name}"</DialogTitle>
      <DialogContent>
        <TextField
          label="Trigger Group Name"
          variant="outlined"
          defaultValue={triggerGroup.name}
          inputRef={nameRef}
          sx={{ mt: 1, mb: 1 }}
          fullWidth
        />
        <TextField
          label="Comment (Optional)"
          variant="outlined"
          defaultValue={triggerGroup.comment}
          inputRef={commentRef}
          sx={{ mt: 1, mb: 1 }}
          fullWidth
          multiline
        />
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onCancel}>
          Cancel
        </Button>
        <Button variant="contained" onClick={saveClicked}>
          Save
        </Button>
      </DialogActions>
    </Dialog>
  );
};

const TriggerListItem: React.FC<{
  triggerID: UUID;
  activeTriggers: Set<string> | null;
}> = ({ triggerID, activeTriggers }) => {
  const dispatch = useDispatch();
  const trigger = useSelector($trigger(triggerID));
  const editingTrigger = useSelector($draftTrigger);
  const activeTriggerTagID = useSelector($activeTriggerTagID);

  const menuContext = useContext(TriggerMenuContext);

  const enabled = !!activeTriggers && activeTriggers.has(triggerID);
  const hasContextMenuOpened = menuContext && menuContext[0] === triggerID;

  const handleTriggerContextMenu = (e: React.MouseEvent<HTMLSpanElement>) => {
    e.preventDefault();
    if (menuContext) {
      const [_, setContextMenu] = menuContext;
      setContextMenu({
        triggerID,
        mouseX: e.clientX + 2,
        mouseY: e.clientY + 2,
      });
    }
  };

  return (
    <li
      className={`view-trigger-list-item ${hasContextMenuOpened ? 'view-trigger-list-item-context-menu-open' : ''}`}
    >
      {!!activeTriggerTagID && (
        <>
          <Switch
            size="small"
            checked={enabled}
            className="toggle-trigger-tag-inclusion-switch"
            onChange={({ target: { checked } }) => {
              if (checked) {
                addTriggerToTag(triggerID, activeTriggerTagID).then((deltas) =>
                  dispatch(applyDeltas(deltas))
                );
              } else {
                removeTriggerFromTag(triggerID, activeTriggerTagID).then(
                  (deltas) => dispatch(applyDeltas(deltas))
                );
              }
            }}
          />{' '}
        </>
      )}
      <a
        href="javascript:void"
        className="view-trigger-list-item-name"
        onContextMenu={handleTriggerContextMenu}
        onClick={() =>
          dispatch(
            editTriggerDraft({
              trigger: cloneDeep(trigger),
              triggerTags: $triggerTagsHavingTrigger(trigger.id)(
                store.getState()
              ),
            })
          )
        }
      >
        {trigger.name}
      </a>
    </li>
  );
};

const TriggerGroupListItem: React.FC<{
  groupID: UUID;
  activeTriggers: Set<string> | null;
}> = ({ groupID, activeTriggers }) => {
  const group = useSelector($triggerGroup(groupID));

  const menuContext = useContext(TriggerGroupMenuContext);

  const handleTriggerGroupContextMenu = (
    e: React.MouseEvent<HTMLSpanElement>
  ) => {
    e.preventDefault();
    if (menuContext) {
      const [_, setContextMenu] = menuContext;
      setContextMenu({
        groupID,
        mouseX: e.clientX + 2,
        mouseY: e.clientY + 2,
      });
    }
  };

  return (
    <li>
      <span
        className="view-trigger-group-name"
        onContextMenu={handleTriggerGroupContextMenu}
      >
        {group.name}
      </span>
      {!!group.children.length && (
        <ul className="view-trigger-group-sublist">
          {group.children.map(({ variant, value: id }) => {
            if (variant === 'T') {
              return (
                <TriggerListItem
                  key={id}
                  triggerID={id}
                  activeTriggers={activeTriggers}
                />
              );
            } else if (variant === 'G') {
              return (
                <TriggerGroupListItem
                  key={id}
                  groupID={id}
                  activeTriggers={activeTriggers}
                />
              );
            }
          })}
        </ul>
      )}
    </li>
  );
};

export default TriggerTree;
