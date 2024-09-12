import {
  ControlPointDuplicateOutlined,
  Edit,
  VerticalAlignBottom,
  VerticalAlignTop,
} from '@mui/icons-material';
import AddBoxOutlinedIcon from '@mui/icons-material/AddBoxOutlined';
import DeleteForeverOutlined from '@mui/icons-material/DeleteForeverOutlined';
import DownloadingIcon from '@mui/icons-material/Downloading';
import {
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Divider,
  FormControl,
  InputLabel,
  ListItemIcon,
  Menu,
  MenuItem,
  Select,
  SelectChangeEvent,
  Switch,
  TextField,
} from '@mui/material';
import Button from '@mui/material/Button';
import { clone, cloneDeep, sortBy, values } from 'lodash';
import React, {
  createContext,
  CSSProperties,
  useContext,
  useRef,
  useState,
} from 'react';
import { useDispatch, useSelector } from 'react-redux';

import openGINATriggerFileDialog from '../dialogs/importGINAFile';
import {
  $triggerDraft,
  editTriggerDraft,
} from '../features/triggers/triggerEditorSlice';
import {
  $activeTriggerTag,
  $activeTriggerTagID,
  $topLevel,
  $trigger,
  $triggerGroup,
  $triggerGroupMaybe,
  $triggerTags,
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

  const activeTriggers: Set<string> | null =
    activeTriggerTag && new Set(activeTriggerTag.triggers);

  // TODO: I should use a BTreeSet on the backend and serialize the tags as an Array
  const triggerTags = sortBy(
    clone(useSelector($triggerTags)),
    (tag) => tag.name
  );

  const currentTriggerTagChanged = (event: SelectChangeEvent) => {
    const value = event.target.value;
    dispatch(activateTriggerTagID(value || null));
  };

  const [triggerContextMenu, setTriggerContextMenu] =
    useState<TriggerContextMenuState>(null);

  const [triggerGroupContextMenu, setTriggerGroupContextMenu] =
    useState<TriggerGroupContextMenuState>(null);

  const closeTriggerContextMenu = () => setTriggerContextMenu(null);

  return (
    <div className="trigger-tree">
      <div id="main-scrollable" style={styleMainScrollable}>
        <p style={{ textAlign: 'right' }}>
          <Button
            size="small"
            variant="contained"
            startIcon={<DownloadingIcon />}
            onClick={() => openGINATriggerFileDialog(dispatch)}
          >
            Import GINA Export
          </Button>
        </p>

        <div>
          {!!triggerTags.length && (
            <>
              <FormControl sx={{ minWidth: 175 }} size="small">
                <InputLabel>Trigger Tag</InputLabel>
                <Select
                  size="small"
                  label="Trigger Tag"
                  onChange={currentTriggerTagChanged}
                  value={activeTriggerTag?.id || ''}
                >
                  {triggerTags.map((tag) => (
                    <MenuItem key={tag.id} value={tag.id}>
                      {tag.name}
                    </MenuItem>
                  ))}
                  <MenuItem key={'none'} value={''}>
                    <em>None</em>
                  </MenuItem>
                </Select>
              </FormControl>{' '}
            </>
          )}
          <TagCreationButton />
        </div>
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
                        activeTriggers={activeTriggers}
                      />
                    ) : (
                      <TriggerGroupListItem
                        key={tgd.value}
                        groupID={tgd.value}
                        activeTriggers={activeTriggers}
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

const TagCreationButton: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const inputRef = useRef<HTMLInputElement>(null);
  const triggerTags = useSelector($triggerTags);
  const [open, setOpen] = useState(false);
  const [waiting, setWaiting] = useState(false);
  const [nameLength, setNameLength] = useState(0);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  if (!open) {
    return (
      <Button
        variant="outlined"
        onClick={() => setOpen(true)}
        startIcon={<AddBoxOutlinedIcon />}
      >
        New Trigger Tag
      </Button>
    );
  }

  const saveClicked = () => {
    if (inputRef.current) {
      setWaiting(true);
      const name = inputRef.current.value.trim();
      createTriggerTag(name).then((deltas) => {
        dispatch(applyDeltas(deltas));
        setOpen(false);
        setWaiting(false);
        setNameLength(0);
        const creation = deltas.find(
          (delta) => delta.variant === 'TriggerTagCreated'
        );
        if (creation) {
          dispatch(activateTriggerTagID(creation.value.id));
        }
      });
    }
  };

  const nameChanged = () => {
    if (inputRef.current) {
      const name = inputRef.current.value.trim();
      setNameLength(name.length);
      if (values(triggerTags).find((tag) => tag.name === name)) {
        setErrorMessage('A tag with this name already exists');
      } else {
        setErrorMessage(null);
      }
    }
  };

  return (
    <div>
      <TextField
        label="Name"
        autoFocus={true}
        helperText={
          errorMessage
            ? errorMessage
            : 'Specify the name of the new Trigger Tag'
        }
        error={!!errorMessage}
        disabled={waiting}
        inputRef={inputRef}
        onChange={nameChanged}
        size="small"
      />{' '}
      <Button
        disabled={waiting || nameLength === 0 || !!errorMessage}
        onClick={saveClicked}
        variant="contained"
      >
        Save
      </Button>{' '}
      <Button
        disabled={waiting}
        onClick={() => setOpen(false)}
        variant="outlined"
      >
        Cancel
      </Button>
    </div>
  );
};

const TriggerListItem: React.FC<{
  triggerID: UUID;
  activeTriggers: Set<string> | null;
}> = ({ triggerID, activeTriggers }) => {
  const dispatch = useDispatch();
  const trigger = useSelector($trigger(triggerID));
  const editingTrigger = useSelector($triggerDraft);
  const activeTriggerTagID = useSelector($activeTriggerTagID);

  const menuContext = useContext(TriggerMenuContext);

  const currentlyEditing = editingTrigger?.id === triggerID;
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
      className={`view-trigger-list-item ${currentlyEditing ? 'view-trigger-list-item-currently-editing' : ''} ${hasContextMenuOpened ? 'view-trigger-list-item-context-menu-open' : ''}`}
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
      <span
        onContextMenu={handleTriggerContextMenu}
        onClick={() => dispatch(editTriggerDraft(cloneDeep(trigger)))}
      >
        {trigger.name}
      </span>
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

const styleMainScrollable: CSSProperties = {
  // position: 'absolute',
  // top: 0,
  // right: 0,
  // left: 0,
  // bottom: 0,
  overflowY: 'scroll',
  overflowX: 'hidden',
  // scrollbarGutter: 'stable',
};

export default TriggerTree;
