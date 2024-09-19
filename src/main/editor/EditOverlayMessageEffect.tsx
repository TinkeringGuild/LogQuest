import { useDispatch, useSelector } from 'react-redux';

import TextField from '@mui/material/TextField';

import {
  EffectVariantOverlayMessage,
  setOverlayMessageTemplate,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import EffectWithOptions from './EffectWithOptions';

const EditOverlayMessageEffect: React.FC<{
  selector: TriggerEditorSelector<EffectVariantOverlayMessage>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const dispatch = useDispatch();
  const { value: tmpl } = useSelector(triggerEditorSelector(selector));
  return (
    <EffectWithOptions
      variant="OverlayMessage"
      help="Shows a message on the Overlay."
      onDelete={onDelete}
    >
      <TextField
        label="Overlay Message (Template)"
        fullWidth
        defaultValue={tmpl}
        className="template-input"
        onBlur={(e) =>
          dispatch(
            setOverlayMessageTemplate({ tmpl: e.target.value, selector })
          )
        }
      />
    </EffectWithOptions>
  );
};

export default EditOverlayMessageEffect;
