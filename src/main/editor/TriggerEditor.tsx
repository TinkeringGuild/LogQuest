import { map as pluck } from 'lodash';
import React, { useEffect, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { Add, CancelOutlined, NavigateNext, Save } from '@mui/icons-material';
import { Button, Stack, TextField, Typography } from '@mui/material';

import {
  $$selectTriggerFilter,
  $$triggerDraftEffects,
  $draftParentPosition,
  $draftTrigger,
  $draftTriggerTags,
  $editorHasError,
  cancelEditing,
  insertNewEffect,
  setTriggerComment,
  setTriggerName,
  setTriggerTags,
} from '../../features/triggers/triggerEditorSlice';
import {
  $groupsUptoGroup,
  $triggerTags,
  applyDeltas,
} from '../../features/triggers/triggersSlice';
import { Timestamp } from '../../generated/Timestamp';
import { createTrigger, saveTrigger } from '../../ipc';
import { calculateTimeAgo } from '../../util';
import StandardTooltip from '../../widgets/StandardTooltip';
import { EffectVariant } from './effect-utils';
import TriggerTagsEditor from './TriggerTagsEditor';
import { createEffectAutocomplete } from './widgets/AutocompleteEffect';
import ControlledTextField from './widgets/ControlledTextField';
import EditFilter from './widgets/EditFilter';
import EffectList from './widgets/EffectList';

import './TriggerEditor.css';

const TriggerEditor: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const trigger = useSelector($draftTrigger);

  const ancestors = useSelector($groupsUptoGroup(trigger.parent_id));

  const triggerTagsOfTrigger = useSelector($draftTriggerTags);
  const allTriggerTags = useSelector($triggerTags);
  const parentPosition = useSelector($draftParentPosition);

  const hasError = useSelector($editorHasError);

  // parentPosition is only needed to be given when creating, so it is also used to signal
  // whether this Trigger is being newly created.
  const isNew = parentPosition !== null;

  const submit = async () => {
    const triggerTagIDs = pluck(triggerTagsOfTrigger, 'id');
    const deltas = isNew
      ? await createTrigger(trigger, triggerTagIDs, parentPosition)
      : await saveTrigger(trigger, triggerTagIDs);
    dispatch(applyDeltas(deltas));
    dispatch(cancelEditing());
  };

  return (
    <div className="trigger-editor scrollable-container">
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
          Last updated <TimeAgo timestamp={trigger.updated_at} />
        </p>

        <div>
          <ControlledTextField
            label="Trigger Name"
            fullWidth
            value={trigger.name}
            slotProps={{ htmlInput: { sx: { fontSize: 30 } } }}
            className="trigger-editor-name"
            validate={(value) => (value.trim() ? null : 'Name cannot be blank')}
            onCommit={(input) => dispatch(setTriggerName(input))}
            autoFocus={trigger.name.trim() === ''}
          />
        </div>

        <div style={{ marginTop: 5 }}>
          <TextField
            multiline
            fullWidth
            placeholder="Comments"
            maxRows={10}
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
          <CreateEffectButton
            create={(variant) => {
              dispatch(
                insertNewEffect({
                  variant,
                  index: 0,
                  triggerID: trigger.id,
                  seqSelector: $$triggerDraftEffects,
                })
              );
            }}
          />
        </div>
        {trigger.effects.length ? (
          <EffectList triggerID={trigger.id} selector={$$triggerDraftEffects} />
        ) : (
          <p>Create a new Effect to execute when one of the Filters matches.</p>
        )}
      </div>
    </div>
  );
};

const TimeAgo: React.FC<{ timestamp: Timestamp }> = ({ timestamp }) => {
  const [[timeAgo, timeExact], setCalculatedTime] = useState(['', '']);

  useEffect(() => {
    setCalculatedTime(calculateTimeAgo(timestamp));

    const interval = setInterval(
      () => setCalculatedTime(calculateTimeAgo(timestamp)),
      1000
    );

    return () => {
      clearInterval(interval);
    };
  }, [timestamp]);

  return (
    <StandardTooltip help={timeExact}>
      <span>{timeAgo} ago</span>
    </StandardTooltip>
  );
};

const CreateEffectButton: React.FC<{
  create: (variant: EffectVariant) => void;
}> = ({ create }) => {
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

  return createEffectAutocomplete({
    onSelect: create,
    close: () => setIsOpen(false),
  });
};

export default TriggerEditor;
