import { TextField, Tooltip } from '@mui/material';
import { formatDistanceToNow } from 'date-fns/formatDistanceToNow';
import { parseISO } from 'date-fns/fp/parseISO';
import React from 'react';
import { useDispatch } from 'react-redux';

import { format } from 'date-fns/fp/format';
import { updateTriggerEffect } from '../features/triggers/triggersSlice';
import { Effect } from '../generated/Effect';
import { EffectWithID } from '../generated/EffectWithID';
import { TemplateString } from '../generated/TemplateString';
import { Trigger } from '../generated/Trigger';
import { UUID } from '../generated/UUID';

import './TriggerEditor.css';

interface TriggerEditorProps {
  trigger: Trigger;
}

const TriggerEditor: React.FC<TriggerEditorProps> = ({ trigger }) => {
  const updatedAt = parseISO(trigger.updated_at);
  const updatedAgo = formatDistanceToNow(updatedAt);
  const updatedExact = format('PPpp', updatedAt);
  return (
    <div className="trigger-editor">
      <h3>{trigger.name}</h3>
      {trigger.comment && <p>{trigger.comment}</p>}
      <p>
        Last updated:{' '}
        <Tooltip arrow title={updatedExact}>
          <span>{updatedAgo} ago</span>
        </Tooltip>
      </p>
      <h4>Effects</h4>
      <div>
        {trigger.effects.map((effect) => {
          return <EditEffect triggerID={trigger.id} effect={effect} />;
        })}
      </div>
    </div>
  );
};

const EditEffect: React.FC<{ triggerID: UUID; effect: EffectWithID }> = ({
  triggerID,
  effect,
}) => {
  switch (effect.inner.variant) {
    case 'CopyToClipboard':
      return (
        <EditCopyToClipboardEffect
          triggerID={triggerID}
          effectID={effect.id}
          tmpl={effect.inner.value}
        />
      );
    case 'Speak':
      return (
        <EditSpeakEffect
          triggerID={triggerID}
          effectID={effect.id}
          tmpl={effect.inner.value.tmpl}
        />
      );
    default:
      return <p>{effect.inner.variant}</p>;
  }
};

const EditSpeakEffect: React.FC<{
  triggerID: UUID;
  effectID: UUID;
  tmpl: TemplateString;
}> = ({ triggerID, effectID, tmpl }) => {
  const dispatch = useDispatch();
  return (
    <div>
      <p className="effect-name">Text-to-Speech</p>
      <TextField
        label="Template"
        variant="standard"
        defaultValue={tmpl}
        onChange={({ target }) => {
          const mutation = (effect: Effect) => {
            if (effect.variant === 'Speak') {
              effect.value.tmpl = target.value;
            }
          };
          dispatch(updateTriggerEffect({ triggerID, effectID, mutation }));
        }}
      />
    </div>
  );
};

const EditCopyToClipboardEffect: React.FC<{
  triggerID: UUID;
  effectID: UUID;
  tmpl: TemplateString;
}> = ({ tmpl }) => {
  return (
    <div>
      <p className="effect-name">Copy to Clipboard</p>
      <input type="text" value={tmpl} />
    </div>
  );
};

export default TriggerEditor;
