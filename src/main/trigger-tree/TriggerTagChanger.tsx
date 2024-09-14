import { clone, sortBy } from 'lodash';
import { useState } from 'react';
import { useSelector } from 'react-redux';

import AddBoxOutlined from '@mui/icons-material/AddBoxOutlined';
import CancelOutlined from '@mui/icons-material/CancelOutlined';
import Save from '@mui/icons-material/Save';
import Button from '@mui/material/Button';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import Select, { SelectChangeEvent } from '@mui/material/Select';
import TextField from '@mui/material/TextField';

import {
  $activeTriggerTag,
  $triggerTags,
} from '../../features/triggers/triggersSlice';
import { TriggerTag } from '../../generated/TriggerTag';
import { UUID } from '../../generated/UUID';

type OnCreateCallback = (triggerTagName: string) => void;
type OnChangeCallback = (triggerTagID: UUID | null) => void;

const TriggerTagChanger: React.FC<{
  onChange: OnChangeCallback;
  onCreate: OnCreateCallback;
}> = ({ onChange, onCreate }) => {
  const activeTriggerTag = useSelector($activeTriggerTag);
  const [mode, setMode] = useState<'select' | 'create'>('select');
  const unsortedTriggerTags = useSelector($triggerTags);

  const triggerTags: TriggerTag[] = sortBy(clone(unsortedTriggerTags), (tag) =>
    tag.name.toUpperCase()
  );

  return mode === 'select' ? (
    <SelectTriggerTagMode
      activeTriggerTagID={activeTriggerTag?.id ?? null}
      triggerTags={triggerTags}
      createClicked={() => setMode('create')}
      onChange={onChange}
    />
  ) : (
    <CreateTriggerTagMode
      unavailableNamesUppercase={triggerTags.map((tag) =>
        tag.name.toUpperCase()
      )}
      onCreate={(name) => {
        onCreate(name);
        setMode('select');
      }}
      onCancel={() => setMode('select')}
    />
  );
};

const SelectTriggerTagMode: React.FC<{
  triggerTags: TriggerTag[];
  activeTriggerTagID: UUID | null;
  createClicked: () => void;
  onChange: OnChangeCallback;
}> = ({ triggerTags, activeTriggerTagID, createClicked, onChange }) => {
  const createButton = <CreateTriggerTagButton onClick={createClicked} />;

  if (triggerTags.length === 0) {
    return <div>{createButton} Create a Trigger Tag to enable Triggers</div>;
  }

  return (
    <div>
      <SelectTriggerTagMenu
        activeTriggerTagID={activeTriggerTagID}
        triggerTags={triggerTags}
        onChange={onChange}
      />{' '}
      {createButton}
    </div>
  );
};

const CreateTriggerTagButton: React.FC<{ onClick: () => void }> = ({
  onClick,
}) => (
  <Button variant="outlined" onClick={onClick} startIcon={<AddBoxOutlined />}>
    New Trigger Tag
  </Button>
);

const SelectTriggerTagMenu: React.FC<{
  activeTriggerTagID: UUID | null;
  triggerTags: TriggerTag[];
  onChange: OnChangeCallback;
}> = ({ activeTriggerTagID, triggerTags, onChange }) => (
  <FormControl sx={{ minWidth: 175 }} size="small">
    <InputLabel>Trigger Tag</InputLabel>
    <Select
      size="small"
      label="Trigger Tag"
      onChange={(event: SelectChangeEvent) => onChange(event.target.value)}
      value={activeTriggerTagID ?? ''}
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
  </FormControl>
);

const CreateTriggerTagMode: React.FC<{
  unavailableNamesUppercase: string[];
  onCreate: OnCreateCallback;
  onCancel: () => void;
}> = ({ unavailableNamesUppercase, onCreate, onCancel }) => {
  const [nameInput, setNameInput] = useState('');

  const errorMessage: string | null = (() => {
    const trimmed = nameInput.trim();
    if (trimmed.length === 0) {
      return 'Trigger Tag name is blank';
    }
    if (unavailableNamesUppercase.includes(trimmed.toUpperCase())) {
      return 'Name already exists';
    }
    return null;
  })();

  return (
    <div>
      <TextField
        label="Name"
        autoFocus={true}
        helperText={errorMessage}
        error={!!errorMessage}
        onChange={(e) => setNameInput(e.target.value)}
        size="small"
      />{' '}
      <Button
        onClick={() => onCreate(nameInput.trim())}
        variant="contained"
        disabled={!!errorMessage}
        startIcon={<Save />}
      >
        Save
      </Button>{' '}
      <Button
        onClick={onCancel}
        variant="outlined"
        startIcon={<CancelOutlined />}
      >
        Cancel
      </Button>
    </div>
  );
};

export default TriggerTagChanger;
