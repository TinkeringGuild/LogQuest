import TextField from '@mui/material/TextField';
import { useDispatch, useSelector } from 'react-redux';
import Card from '@mui/material/Card';
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent';

import {
  triggerEditorSelector,
  TriggerEditorSelector,
  EffectVariantSpeak,
  setSpeakTemplate,
} from '../../features/triggers/triggerEditorSlice';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';
import { RecordVoiceOverOutlined } from '@mui/icons-material';
import { useRef } from 'react';
import Checkbox from '@mui/material/Checkbox';
import FormGroup from '@mui/material/FormGroup';
import FormControlLabel from '@mui/material/FormControlLabel';

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
    <Card elevation={10}>
      <CardHeader
        title={
          <EffectHeader onDelete={onDelete}>
            <EffectTitle
              title="Speak"
              help="Uses the system Text-to-Speech engine to speak the (templated) text"
              icon={<RecordVoiceOverOutlined />}
            />
          </EffectHeader>
        }
      />
      <CardContent>
        <TextField
          label="Text-to-Speech Text (Template)"
          defaultValue={tmpl}
          fullWidth
          inputRef={tmplRef}
          onBlur={dispatchUpdate}
        />
        <FormGroup>
          <FormControlLabel
            label="Interrupt other Text-to-Speech playback"
            control={
              <Checkbox
                inputRef={interruptRef}
                defaultChecked={interrupt}
                onChange={dispatchUpdate}
              />
            }
          />
        </FormGroup>
      </CardContent>
    </Card>
  );
};

export default EditSpeakEffect;
