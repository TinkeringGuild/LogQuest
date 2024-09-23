import { clone, sortBy } from 'lodash';
import { useEffect, useId, useRef, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import AddBoxSharp from '@mui/icons-material/AddBoxSharp';
import DeleteForeverSharp from '@mui/icons-material/DeleteForeverSharp';
import DisabledByDefaultSharp from '@mui/icons-material/DisabledByDefaultSharp';
import Edit from '@mui/icons-material/Edit';
import Info from '@mui/icons-material/Info';
import LibraryAddSharp from '@mui/icons-material/LibraryAddSharp';
import More from '@mui/icons-material/More';
import Button from '@mui/material/Button';
import ButtonGroup from '@mui/material/ButtonGroup';
import FormControl from '@mui/material/FormControl';
import IconButton from '@mui/material/IconButton';
import InputLabel from '@mui/material/InputLabel';
import ListItemIcon from '@mui/material/ListItemIcon';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import Select, { SelectChangeEvent } from '@mui/material/Select';
import TextField from '@mui/material/TextField';
import PopupState, { bindMenu, bindTrigger } from 'material-ui-popup-state';

import { updateActivedTriggerTagIDs } from '../../features/app/appSlice';
import {
  $activeTriggerTag,
  $triggerTags,
  activateTriggerTagID,
  applyDeltas,
} from '../../features/triggers/triggersSlice';
import { TriggerTag } from '../../generated/TriggerTag';
import { UUID } from '../../generated/UUID';
import { deleteTriggerTag, renameTriggerTag } from '../../ipc';
import StandardTooltip from '../../widgets/StandardTooltip';

import './TriggerTagChanger.css';

type OnCreateCallback = (triggerTagName: string) => void;
type OnRenameCallback = (triggerTagName: string) => void;
type OnChangeCallback = (triggerTagID: UUID | null) => void;

const TriggerTagChanger: React.FC<{
  onChange: OnChangeCallback;
  onCreate: OnCreateCallback;
}> = ({ onChange, onCreate }) => {
  const dispatch = useDispatch();

  const activeTriggerTag = useSelector($activeTriggerTag);
  const [mode, setMode] = useState<'select' | 'create' | 'rename'>('select');
  const unsortedTriggerTags = useSelector($triggerTags);

  const triggerTags: TriggerTag[] = sortBy(clone(unsortedTriggerTags), (tag) =>
    tag.name.toUpperCase()
  );

  return mode === 'select' ? (
    <SelectTriggerTagMode
      activeTriggerTagID={activeTriggerTag?.id ?? null}
      triggerTags={triggerTags}
      onCreateClicked={() => setMode('create')}
      onRenameClicked={() => setMode('rename')}
      onChange={onChange}
    />
  ) : (
    <TriggerTagNameEntryMode
      mode={mode}
      activeTriggerTagName={activeTriggerTag?.name}
      unavailableNamesUppercase={triggerTags.map((tag) =>
        tag.name.toUpperCase()
      )}
      onCreate={(name) => {
        onCreate(name);
        setMode('select');
      }}
      onRename={async (name) => {
        if (!activeTriggerTag) return;
        const deltas = await renameTriggerTag(activeTriggerTag.id, name);
        dispatch(applyDeltas(deltas));
        setMode('select');
      }}
      onCancel={() => setMode('select')}
    />
  );
};

const SelectTriggerTagMode: React.FC<{
  triggerTags: TriggerTag[];
  activeTriggerTagID: UUID | null;
  onCreateClicked: () => void;
  onRenameClicked: () => void;
  onChange: OnChangeCallback;
}> = ({
  triggerTags,
  activeTriggerTagID,
  onCreateClicked,
  onRenameClicked,
  onChange,
}) => {
  const dispatch = useDispatch();

  if (triggerTags.length === 0) {
    return (
      <div>
        <Button
          variant={triggerTags.length === 0 ? 'contained' : 'outlined'}
          onClick={onCreateClicked}
          startIcon={<Info />}
        ></Button>
        Create a Trigger Tag to enable Triggers
      </div>
    );
  }

  const deleteActiveTriggerTag = async () => {
    if (!activeTriggerTagID) return;
    const { activeTriggerTags, deltas } =
      await deleteTriggerTag(activeTriggerTagID);
    dispatch(activateTriggerTagID(null));
    dispatch(updateActivedTriggerTagIDs(activeTriggerTags));
    dispatch(applyDeltas(deltas));
  };

  return (
    <div>
      <SelectTriggerTagMenu
        activeTriggerTagID={activeTriggerTagID}
        triggerTags={triggerTags}
        onChange={onChange}
        onCreateClicked={onCreateClicked}
        onRenameClicked={onRenameClicked}
        onDeleteClicked={deleteActiveTriggerTag}
      />{' '}
    </div>
  );
};

const SelectTriggerTagMenu: React.FC<{
  activeTriggerTagID: UUID | null;
  triggerTags: TriggerTag[];
  onChange: OnChangeCallback;
  onCreateClicked: () => void;
  onRenameClicked: () => void;
  onDeleteClicked: () => void;
}> = ({
  activeTriggerTagID,
  triggerTags,
  onChange,
  onCreateClicked,
  onRenameClicked,
  onDeleteClicked,
}) => {
  const moreMenuID = useId();

  return (
    <FormControl sx={{ minWidth: 175 }} size="small">
      <InputLabel>Trigger Tag</InputLabel>
      <ButtonGroup>
        <Select
          size="small"
          label="Trigger Tag"
          id="trigger-tag-select"
          onChange={(event: SelectChangeEvent) => onChange(event.target.value)}
          value={activeTriggerTagID ?? ''}
          sx={{ width: 200 }}
        >
          <MenuItem
            key={'none'}
            value={''}
            className="trigger-tag-select-option-none"
          >
            None
          </MenuItem>
          {triggerTags.map((tag) => (
            <MenuItem key={tag.id} value={tag.id}>
              {tag.name}
            </MenuItem>
          ))}
        </Select>
        <PopupState variant="popover" popupId={moreMenuID}>
          {(popupState) => (
            <>
              <Button
                startIcon={<More />}
                className="trigger-tag-changer-more-button"
                sx={{ color: 'black' }}
                {...bindTrigger(popupState)}
              />
              <Menu
                {...bindMenu(popupState)}
                transformOrigin={{
                  vertical: 'top',
                  horizontal: 'right',
                }}
                anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
              >
                <MenuItem
                  onClick={() => {
                    popupState.close();
                    onCreateClicked();
                  }}
                >
                  <ListItemIcon>
                    <AddBoxSharp />
                  </ListItemIcon>
                  Create Trigger Tag
                </MenuItem>
                <MenuItem onClick={popupState.close} disabled={true}>
                  <ListItemIcon>
                    <LibraryAddSharp />
                  </ListItemIcon>
                  Duplicate
                </MenuItem>
                <MenuItem
                  onClick={() => {
                    popupState.close();
                    onRenameClicked();
                  }}
                  disabled={!activeTriggerTagID}
                >
                  <ListItemIcon>
                    <Edit />
                  </ListItemIcon>
                  Rename
                </MenuItem>
                <MenuItem
                  onClick={() => {
                    popupState.close();
                    onDeleteClicked();
                  }}
                  disabled={!activeTriggerTagID}
                >
                  <ListItemIcon>
                    <DeleteForeverSharp />
                  </ListItemIcon>
                  Delete
                </MenuItem>
              </Menu>
            </>
          )}
        </PopupState>
      </ButtonGroup>
    </FormControl>
  );
};

const TriggerTagNameEntryMode: React.FC<{
  mode: 'create' | 'rename';
  activeTriggerTagName: string | undefined;
  unavailableNamesUppercase: string[];
  onCreate: OnCreateCallback;
  onRename: OnRenameCallback;
  onCancel: () => void;
}> = ({
  mode,
  activeTriggerTagName,
  unavailableNamesUppercase,
  onCreate,
  onRename,
  onCancel,
}) => {
  const [nameInput, setNameInput] = useState('');
  const nameInputRef = useRef<HTMLInputElement>(null);

  const errorMessage: string | null = (() => {
    const trimmed = nameInput.trim();
    if (trimmed.length === 0) {
      return 'Trigger Tag name is blank';
    }
    if (nameInput === activeTriggerTagName) {
      return 'Enter a new name';
    }
    if (unavailableNamesUppercase.includes(trimmed.toUpperCase())) {
      return 'Name already exists';
    }
    return null;
  })();

  useEffect(() => {
    if (mode == 'rename' && activeTriggerTagName) {
      setNameInput(activeTriggerTagName);
    }
  }, [activeTriggerTagName]);

  useEffect(() => {
    if (mode === 'rename' && nameInputRef.current) {
      nameInputRef.current.select();
    }
  }, []);

  return (
    <div>
      <TextField
        label={mode == 'create' ? 'New Trigger Tag name' : 'Rename Trigger Tag'}
        autoFocus={true}
        helperText={errorMessage}
        error={!!errorMessage}
        defaultValue={
          mode === 'rename' && activeTriggerTagName ? activeTriggerTagName : ''
        }
        inputRef={nameInputRef}
        onChange={(e) => setNameInput(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && !errorMessage) {
            if (mode === 'create') {
              onCreate(nameInput.trim());
            } else {
              onRename(nameInput.trim());
            }
          } else if (e.key === 'Escape') {
            onCancel();
          }
        }}
        size="small"
      />{' '}
      <Button
        onClick={() =>
          mode === 'create'
            ? onCreate(nameInput.trim())
            : onRename(nameInput.trim())
        }
        variant="contained"
        disabled={!!errorMessage}
        startIcon={mode === 'create' ? <AddBoxSharp /> : <AddBoxSharp />}
      >
        {mode === 'create' ? 'Create' : 'Rename'}
      </Button>{' '}
      <StandardTooltip help="Cancel" placement="right">
        <IconButton onClick={onCancel}>
          <DisabledByDefaultSharp />
        </IconButton>
      </StandardTooltip>
    </div>
  );
};

export default TriggerTagChanger;
