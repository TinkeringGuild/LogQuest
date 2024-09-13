import { map as pluck, sortBy, without } from 'lodash';
import { useState } from 'react';

import CancelOutlined from '@mui/icons-material/CancelOutlined';
import CheckCircleOutline from '@mui/icons-material/CheckCircleOutline';
import RadioButtonUnchecked from '@mui/icons-material/RadioButtonUnchecked';
import Save from '@mui/icons-material/Save';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';

import { TriggerTag } from '../../generated/TriggerTag';
import { UUID } from '../../generated/UUID';

const TriggerTagsEditor: React.FC<{
  tagsOfTrigger: TriggerTag[];
  allTriggerTags: TriggerTag[];
  setTriggerTags: (tags: TriggerTag[]) => void;
}> = ({ tagsOfTrigger, allTriggerTags, setTriggerTags }) => {
  const [inEditMode, setInEditMode] = useState(false);

  if (allTriggerTags.length === 0) {
    return (
      <div>
        <p>No Trigger Tags have been created yet.</p>
      </div>
    );
  }

  if (inEditMode) {
    return (
      <EditMode
        current={tagsOfTrigger}
        all={allTriggerTags}
        onSave={(ids) => {
          const selectedTags = [...ids].map(
            (id) => allTriggerTags.find((tag) => tag.id == id)!
          );
          setTriggerTags(sortBy(selectedTags, (tag) => tag.name));
          setInEditMode(false);
        }}
        onCancel={() => setInEditMode(false)}
      />
    );
  } else {
    return (
      <div>
        <div>
          {!tagsOfTrigger.length ? (
            <p>This Trigger has no Trigger Tags</p>
          ) : (
            tagsOfTrigger.map((tag) => (
              <span key={tag.id}>
                <Chip variant={'filled'} label={tag.name} />{' '}
              </span>
            ))
          )}
        </div>{' '}
        <div>
          <Button onClick={() => setInEditMode(true)}>Edit Trigger Tags</Button>
        </div>
      </div>
    );
  }
};

const EditMode: React.FC<{
  current: TriggerTag[];
  all: TriggerTag[];
  onSave: (ids: Set<UUID>) => void;
  onCancel: () => void;
}> = ({ current, all, onSave, onCancel }) => {
  const [selectedIDs, setSelectedIDs] = useState<Set<UUID>>(
    new Set(pluck(current, 'id'))
  );

  const selectID = (id: UUID) => {
    setSelectedIDs(new Set([...selectedIDs, id]));
  };

  const unselectID = (id: UUID) => {
    setSelectedIDs(new Set(without([...selectedIDs], id)));
  };

  return (
    <Stack gap={2}>
      <div>
        {all.map((tag) => {
          const isSelected = selectedIDs.has(tag.id);
          return (
            <span key={tag.id}>
              {isSelected ? (
                <Chip
                  label={tag.name}
                  variant="filled"
                  onClick={() => unselectID(tag.id)}
                  icon={<CheckCircleOutline />}
                  sx={sxRippleColor('white')}
                />
              ) : (
                <Chip
                  label={tag.name}
                  variant="outlined"
                  onClick={() => selectID(tag.id)}
                  icon={<RadioButtonUnchecked />}
                  sx={sxRippleColor(DEFAULT_CHIP_BACKGROUND_COLOR)}
                />
              )}{' '}
            </span>
          );
        })}{' '}
      </div>
      <div>
        <Button
          variant="contained"
          size="small"
          startIcon={<Save />}
          onClick={() => onSave(selectedIDs)}
        >
          Update Trigger Tags
        </Button>{' '}
        <Button
          variant="outlined"
          size="small"
          onClick={onCancel}
          startIcon={<CancelOutlined />}
        >
          Cancel
        </Button>
      </div>
    </Stack>
  );
};

const DEFAULT_CHIP_BACKGROUND_COLOR = 'rgba(0, 0, 0, 0.08)';

const sxRippleColor = (color: string) => ({
  '& .MuiTouchRipple-root': { color },
});

export default TriggerTagsEditor;
