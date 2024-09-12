import { CancelOutlined, Save } from '@mui/icons-material';
import { Button, Stack, TextField, Tooltip } from '@mui/material';
import { formatDistanceToNow } from 'date-fns/formatDistanceToNow';
import { format } from 'date-fns/fp/format';
import { parseISO } from 'date-fns/fp/parseISO';
import React from 'react';
import { useDispatch, useSelector } from 'react-redux';

import {
  $$selectTriggerFilter,
  $$triggerDraftEffects,
  $triggerDraft,
  cancelEditing,
  deleteEffect,
  setTriggerComment,
  setTriggerName,
} from '../../features/triggers/editorSlice';
import { EffectWithID } from '../../generated/EffectWithID';
import EditEffect from './EditEffect';
import EditFilter from './widgets/EditFilter';
import { saveTrigger } from '../../ipc';
import { applyDeltas } from '../../features/triggers/triggersSlice';

import './TriggerEditor.css';

const TriggerEditor: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const trigger = useSelector($triggerDraft);

  if (!trigger) {
    throw new Error(
      'Attempted to render TriggerEditor while not currently editing a Trigger!'
    );
  }

  const updatedAt = parseISO(trigger.updated_at);
  const updatedAgo = formatDistanceToNow(updatedAt);
  const updatedExact = format('PPpp', updatedAt);

  return (
    <div className="trigger-editor">
      <div style={{ marginBottom: 10 }}>
        <Button
          variant="contained"
          size="large"
          startIcon={<Save />}
          onClick={() => {
            // TODO: I need to handle CreateTrigger for when the Trigger is new
            saveTrigger(trigger).then((deltas) => {
              console.log('DELTAS:', deltas);
              dispatch(applyDeltas(deltas));
              dispatch(cancelEditing());
            });
          }}
        >
          Save
        </Button>{' '}
        <Button
          variant="outlined"
          size="large"
          startIcon={<CancelOutlined />}
          onClick={() => dispatch(cancelEditing())}
        >
          Cancel
        </Button>{' '}
      </div>
      <p className="trigger-editor-info">
        Last updated:{' '}
        <Tooltip arrow title={updatedExact}>
          <span>{updatedAgo} ago</span>
        </Tooltip>
      </p>
      <div>
        <TextField
          label="Trigger Name"
          fullWidth
          defaultValue={trigger.name}
          className="trigger-editor-name"
          onBlur={(e) => dispatch(setTriggerName(e.target.value))}
        />
      </div>
      <div style={{ marginTop: 5 }}>
        <TextField
          multiline
          fullWidth
          placeholder="Comments"
          defaultValue={trigger.comment || ''}
          onBlur={(e) => dispatch(setTriggerComment(e.target.value))}
        />
      </div>
      <h3>Filters</h3>
      {trigger.filter.length > 1 && (
        <p className="trigger-editor-info">
          The Trigger will fire if <em>any</em> of the following patterns match.
        </p>
      )}
      <div>
        <EditFilter selector={$$selectTriggerFilter} />
      </div>
      <h3>Effects</h3>
      <Stack gap={2}>
        {trigger.effects.map((effect) => {
          return (
            <EditEffect
              key={effect.id}
              triggerID={trigger.id}
              onDelete={() =>
                dispatch(
                  deleteEffect({
                    effectID: effect.id,
                    selector: $$triggerDraftEffects,
                  })
                )
              }
              effectSelector={(slice) => {
                const draft = slice.draft!;
                const draftEffect = draft.effects.find(
                  ({ id }) => id === effect.id
                ) as EffectWithID;
                return draftEffect!;
              }}
            />
          );
        })}
      </Stack>
    </div>
  );
};

export default TriggerEditor;
