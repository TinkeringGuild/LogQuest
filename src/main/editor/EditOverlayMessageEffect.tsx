import { InsertCommentOutlined } from '@mui/icons-material';
import TextField from '@mui/material/TextField';
import { useDispatch, useSelector } from 'react-redux';

import {
  triggerEditorSelector,
  TriggerEditorSelector,
  EffectVariantOverlayMessage,
  setOverlayMessageTemplate,
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
      title="Show Overlay Message"
      help="Shows a message on the Overlay."
      icon={<InsertCommentOutlined />}
      onDelete={onDelete}
    >
      <TextField
        label="Overlay Message (Template)"
        fullWidth
        defaultValue={tmpl}
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
