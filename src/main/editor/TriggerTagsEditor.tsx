import { map as pluck, sortBy, without } from 'lodash';
import { useState } from 'react';

import CancelOutlined from '@mui/icons-material/CancelOutlined';
import CheckCircleOutline from '@mui/icons-material/CheckCircleOutline';
import RadioButtonUnchecked from '@mui/icons-material/RadioButtonUnchecked';
import Save from '@mui/icons-material/Save';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';

import { TriggerTag } from '../../generated/TriggerTag';
import { UUID } from '../../generated/UUID';

import './TriggerTagsEditor.css';

const TriggerTagsEditor: React.FC<{
  tagsOfTrigger: TriggerTag[];
  allTriggerTags: TriggerTag[];
  onSave: (tags: TriggerTag[]) => void;
}> = ({ tagsOfTrigger, allTriggerTags, onSave }) => {
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
          onSave(sortBy(selectedTags, (tag) => tag.name.toUpperCase()));
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
            <Alert severity="warning">
              No Trigger Tags are assigned to this Trigger.
            </Alert>
          ) : (
            tagsOfTrigger.map((tag) => (
              <span className="trigger-tag-chip" key={tag.id}>
                <Chip color="primary" variant="filled" label={tag.name} />
              </span>
            ))
          )}
        </div>
        <div className="edit-trigger-tags-button-container">
          <Button variant="outlined" onClick={() => setInEditMode(true)}>
            Edit Trigger Tags
          </Button>
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

  const allSorted = sortBy(all, (tag) => tag.name.toUpperCase());

  return (
    <Stack gap={2}>
      <div>
        {allSorted.map((tag) => {
          const isSelected = selectedIDs.has(tag.id);
          return (
            <span key={tag.id} className="trigger-tag-chip">
              {isSelected ? (
                <Chip
                  label={tag.name}
                  variant="filled"
                  color="primary"
                  onClick={() => unselectID(tag.id)}
                  icon={<CheckCircleOutline />}
                />
              ) : (
                <Chip
                  label={tag.name}
                  variant="outlined"
                  onClick={() => selectID(tag.id)}
                  icon={<RadioButtonUnchecked />}
                />
              )}
            </span>
          );
        })}
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

export default TriggerTagsEditor;
