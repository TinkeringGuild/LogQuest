import { InsertCommentOutlined } from '@mui/icons-material';
import TextField from '@mui/material/TextField';
import { useDispatch, useSelector } from 'react-redux';

import {
  editorSelector,
  EditorSelector,
  EffectVariantOverlayMessage,
  setOverlayMessageTemplate,
} from '../../features/triggers/editorSlice';
import EffectWithOptions from './EffectWithOptions';

const EditOverlayMessageEffect: React.FC<{
  selector: EditorSelector<EffectVariantOverlayMessage>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const dispatch = useDispatch();
  const { value: tmpl } = useSelector(editorSelector(selector));
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
