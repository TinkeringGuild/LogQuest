import { formatDistanceToNow } from 'date-fns/formatDistanceToNow';
import { format } from 'date-fns/fp/format';
import { parseISO } from 'date-fns/fp/parseISO';
import { map as pluck } from 'lodash';
import React, { useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { Add, CancelOutlined, NavigateNext, Save } from '@mui/icons-material';
import { Button, Stack, TextField, Tooltip, Typography } from '@mui/material';

import {
  $$selectTriggerFilter,
  $$triggerDraftEffects,
  $draftParentPosition,
  $draftTrigger,
  $draftTriggerTags,
  $editorHasError,
  cancelEditing,
  deleteEffect,
  insertEffect,
  setTriggerComment,
  setTriggerName,
  setTriggerTags,
} from '../../features/triggers/triggerEditorSlice';
import {
  $groupsUptoGroup,
  $triggerTags,
  applyDeltas,
} from '../../features/triggers/triggersSlice';
import { UUID } from '../../generated/UUID';
import { createTrigger, saveTrigger } from '../../ipc';
import EditEffect from './EditEffect';
import TriggerTagsEditor from './TriggerTagsEditor';
import AutocompleteEffect from './widgets/AutocompleteEffect';
import EditFilter from './widgets/EditFilter';

import './TriggerEditor.css';

const TriggerEditor: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const trigger = useSelector($draftTrigger);

  const ancestors = useSelector($groupsUptoGroup(trigger.parent_id));

  const triggerTagsOfTrigger = useSelector($draftTriggerTags);
  const allTriggerTags = useSelector($triggerTags);
  const parentPosition = useSelector($draftParentPosition);

  const editorHasError = useSelector($editorHasError);
  const [nameError, setNameError] = useState<string | undefined>(undefined);

  const hasError = editorHasError || !!nameError;

  // parentPosition is only needed to be given when creating, so it is also used to signal
  // whether this Trigger is being newly created.
  const isNew = parentPosition !== null;

  const updatedAt = parseISO(trigger.updated_at);
  const updatedAgo = formatDistanceToNow(updatedAt);
  const updatedExact = format('PPpp', updatedAt);

  const submit = async () => {
    const triggerTagIDs = pluck(triggerTagsOfTrigger, 'id');
    const deltas = isNew
      ? await createTrigger(trigger, triggerTagIDs, parentPosition)
      : await saveTrigger(trigger, triggerTagIDs);
    dispatch(applyDeltas(deltas));
    dispatch(cancelEditing());
  };

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
            disabled={hasError}
            startIcon={<Save />}
            onClick={submit}
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
            error={!!nameError}
            slotProps={{ htmlInput: { sx: { fontSize: 30 } } }}
            helperText={nameError}
            className="trigger-editor-name"
            onChange={(e) => {
              e.target.value.trim()
                ? nameError !== undefined && setNameError(undefined)
                : setNameError('Name cannot be blank');
            }}
            onBlur={(e) => dispatch(setTriggerName(e.target.value))}
            autoFocus={trigger.name.trim() === ''}
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
            onSave={(tags) => dispatch(setTriggerTags(tags))}
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
        <h3 style={{ marginBottom: 10 }}>Effects</h3>
        <div style={{ height: 45, marginBottom: 15 }}>
          <CreateEffectButton triggerID={trigger.id} />
        </div>
        {!!trigger.effects.length ? (
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
                  effectSelector={({ draft }) =>
                    draft!.effects.find(({ id }) => id === effect.id)!
                  }
                />
              );
            })}
          </Stack>
        ) : (
          <p>Create a new Effect to execute when one of the Filters matches.</p>
        )}
      </div>
    </div>
  );
};

const CreateEffectButton: React.FC<{ triggerID: UUID }> = ({ triggerID }) => {
  const dispatch = useDispatch();
  const [isOpen, setIsOpen] = useState(false);

  if (!isOpen) {
    return (
      <Button
        variant="contained"
        size="large"
        startIcon={<Add />}
        onClick={() => setIsOpen(true)}
        sx={{ width: 200 }}
      >
        Add New Effect
      </Button>
    );
  }

  return (
    <AutocompleteEffect
      close={() => setIsOpen(false)}
      onSelect={(variant) => {
        dispatch(
          insertEffect({
            variant,
            index: 0,
            triggerID: triggerID,
            seqSelector: $$triggerDraftEffects,
          })
        );
      }}
    />
  );
};

export default TriggerEditor;
