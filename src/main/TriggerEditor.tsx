import React from 'react';
import { formatDistanceToNow } from 'date-fns/formatDistanceToNow';
import { parseISO } from 'date-fns/fp/parseISO';
import { TextField, Tooltip } from '@mui/material';

import { Trigger } from '../generated/Trigger';
import { format } from 'date-fns/fp/format';
import { Effect } from '../generated/Effect';
import { TemplateString } from '../generated/TemplateString';

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
          return <EditEffect effect={effect} />;
        })}
      </div>
    </div>
  );
};

const EditEffect: React.FC<{ effect: Effect }> = ({ effect }) => {
  switch (effect.variant) {
    case 'CopyToClipboard':
      return <EditCopyToClipboardEffect tmpl={effect.value} />;
    case 'Speak':
      return <EditSpeakEffect tmpl={effect.value.tmpl} />;
    default:
      return <p>{effect.variant}</p>;
  }
};

const EditSpeakEffect: React.FC<{ tmpl: TemplateString }> = ({ tmpl }) => {
  return (
    <div>
      <p className="effect-name">Text-to-Speech</p>
      <TextField label="Template" variant="standard" value={tmpl.tmpl} />
    </div>
  );
};

const EditCopyToClipboardEffect: React.FC<{ tmpl: TemplateString }> = ({
  tmpl,
}) => {
  return (
    <div>
      <p className="effect-name">Copy to Clipboard</p>
      <input type="text" value={tmpl.tmpl} />
    </div>
  );
};

export default TriggerEditor;
