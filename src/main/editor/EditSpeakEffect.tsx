import { useRef } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import Checkbox from '@mui/material/Checkbox';
import FormControlLabel from '@mui/material/FormControlLabel';
import FormGroup from '@mui/material/FormGroup';
import TextField from '@mui/material/TextField';

import {
  EffectVariantSpeak,
  setSpeakTemplate,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import EffectWithOptions from './EffectWithOptions';

const EditSpeakEffect: React.FC<{
  selector: TriggerEditorSelector<EffectVariantSpeak>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const dispatch = useDispatch();
  const {
    value: { tmpl, interrupt },
  } = useSelector(triggerEditorSelector(selector));
  const tmplRef = useRef<HTMLInputElement>(null);
  const interruptRef = useRef<HTMLInputElement>(null);

  const dispatchUpdate = () => {
    if (tmplRef.current && interruptRef.current) {
      const tmpl = tmplRef.current.value;
      const interrupt = interruptRef.current.checked;
      dispatch(setSpeakTemplate({ tmpl, interrupt, selector }));
    }
  };

  return (
    <EffectWithOptions
      variant="Speak"
      help="Uses the system Text-to-Speech engine to speak the (templated) text"
      onDelete={onDelete}
    >
      <TextField
        label="Text-to-Speech Text (Template)"
        defaultValue={tmpl}
        fullWidth
        className="template-input"
        inputRef={tmplRef}
        onBlur={dispatchUpdate}
      />
      <FormGroup>
        <FormControlLabel
          label="Interrupts other Text-to-Speech playback"
          control={
            <Checkbox
              inputRef={interruptRef}
              defaultChecked={interrupt}
              onChange={dispatchUpdate}
            />
          }
        />
      </FormGroup>
    </EffectWithOptions>
  );
};

export default EditSpeakEffect;
