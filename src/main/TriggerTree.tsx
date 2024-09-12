import AddBoxOutlinedIcon from '@mui/icons-material/AddBoxOutlined';
import DownloadingIcon from '@mui/icons-material/Downloading';
import {
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  SelectChangeEvent,
  Switch,
  TextField,
} from '@mui/material';
import Button from '@mui/material/Button';
import { clone, cloneDeep, sortBy, values } from 'lodash';
import React, { CSSProperties, useRef, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import openGINATriggerFileDialog from '../dialogs/importGINAFile';
import {
  $triggerDraft,
  editTriggerDraft,
} from '../features/triggers/editorSlice';
import {
  $activeTriggerTag,
  $activeTriggerTagID,
  $topLevel,
  $trigger,
  $triggerGroup,
  $triggerTags,
  activateTriggerTagID,
  applyDeltas,
} from '../features/triggers/triggersSlice';
import { TriggerGroupDescendant } from '../generated/TriggerGroupDescendant';
import { UUID } from '../generated/UUID';
import {
  addTriggerToTag,
  createTriggerTag,
  removeTriggerFromTag,
} from '../ipc';

import './TriggerTree.css';

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
      </div>
    </div>
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

  const selected = editingTrigger?.id === triggerID;
  const enabled = !!activeTriggers && activeTriggers.has(triggerID);

  return (
    <li
      className={`view-trigger-list-item ${selected ? 'view-trigger-list-item-selected' : ''}`}
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
      <span onClick={() => dispatch(editTriggerDraft(cloneDeep(trigger)))}>
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

  return (
    <li>
      <span className="view-trigger-group-name">{group.name}</span>
      {group.children.length && (
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
