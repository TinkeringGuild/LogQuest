import { formatDistanceToNow } from 'date-fns/formatDistanceToNow';
import { format } from 'date-fns/fp/format';
import { parseISO } from 'date-fns/fp/parseISO';
import React from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { CancelOutlined, NavigateNext, Save } from '@mui/icons-material';
import { Button, Stack, TextField, Tooltip, Typography } from '@mui/material';

import {
  $$selectTriggerFilter,
  $$triggerDraftEffects,
  $draftTrigger,
  $draftTriggerTags,
  cancelEditing,
  deleteEffect,
  setTriggerComment,
  setTriggerName,
  setTriggerTags,
} from '../../features/triggers/triggerEditorSlice';
import {
  $ancestorGroupsForTriggerID,
  $triggerTags,
  applyDeltas,
} from '../../features/triggers/triggersSlice';
import { EffectWithID } from '../../generated/EffectWithID';
import { saveTrigger } from '../../ipc';
import EditEffect from './EditEffect';
import TriggerTagsEditor from './TriggerTagsEditor';
import EditFilter from './widgets/EditFilter';

import './TriggerEditor.css';

const TriggerEditor: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const trigger = useSelector($draftTrigger);

  const ancestors = useSelector($ancestorGroupsForTriggerID(trigger.id));

  if (!trigger) {
    throw new Error(
      'Attempted to render TriggerEditor while not currently editing a Trigger!'
    );
  }

  const triggerTagsOfTrigger = useSelector($draftTriggerTags);
  const allTriggerTags = useSelector($triggerTags);

  const updatedAt = parseISO(trigger.updated_at);
  const updatedAgo = formatDistanceToNow(updatedAt);
  const updatedExact = format('PPpp', updatedAt);

  return (
    <div className="trigger-editor trigger-browser-scrollable-container">
      <div className="trigger-browser-scrollable-content scrollable-content central-content">
        <div className="trigger-editor-breadcrumbs">
          <Stack
            direction="row"
            divider={<NavigateNext />}
            alignItems="center"
            justifyContent="center"
          >
            {ancestors.map((ancestor) => (
              <Typography key={ancestor.id} fontWeight="bold">
                {ancestor.name}
              </Typography>
            ))}
            <Typography>{trigger.name}</Typography>
          </Stack>
        </div>
        <div style={{ marginBottom: 10, textAlign: 'center' }}>
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
        <p className="trigger-editor-info" style={{ textAlign: 'center' }}>
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
        <div style={{ marginTop: 10, marginBottom: 20 }}>
          <h3 className="trigger-tags-header">Trigger Tags</h3>
          <TriggerTagsEditor
            tagsOfTrigger={triggerTagsOfTrigger}
            allTriggerTags={Object.values(allTriggerTags)}
            setTriggerTags={(tags) => dispatch(setTriggerTags(tags))}
          />
        </div>
        <h3>Filters</h3>
        {trigger.filter.length > 1 && (
          <p className="trigger-editor-info">
            The Trigger will fire if <em>any</em> of the following patterns
            match.
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
    </div>
  );
};

export default TriggerEditor;
