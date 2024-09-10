import TextField from '@mui/material/TextField';
import { useDispatch } from 'react-redux';

import { updateTriggerEffect } from '../../features/triggers/triggersSlice';
import { Effect } from '../../generated/Effect';
import { TemplateString } from '../../generated/TemplateString';
import { UUID } from '../../generated/UUID';

const EditSpeakEffect: React.FC<{
  triggerID: UUID;
  effectID: UUID;
  tmpl: TemplateString;
  onDelete: () => void;
}> = ({ triggerID, effectID, tmpl, onDelete: _TODO }) => {
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

export default EditSpeakEffect;
