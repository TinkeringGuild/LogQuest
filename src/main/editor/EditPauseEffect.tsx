import { PauseCircleOutline } from '@mui/icons-material';
import TextField from '@mui/material/TextField';
import { debounce } from 'lodash';
import { useDispatch, useSelector } from 'react-redux';

import {
  editorSelector,
  EditorSelector,
  EffectVariantPause,
  setPauseDuration,
} from '../../features/triggers/editorSlice';
import EffectWithOptions from './EffectWithOptions';

const DEBOUNCE_WAIT_MILLIS = 300;

const EditPauseEffect: React.FC<{
  selector: EditorSelector<EffectVariantPause>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const dispatch = useDispatch();
  const { value: millis } = useSelector(editorSelector(selector));

  const triggerChange = (seconds: number) => {
    dispatch(setPauseDuration({ millis: seconds * 1000, selector }));
  };

  const triggerChangeDebounced = debounce(triggerChange, DEBOUNCE_WAIT_MILLIS);

  return (
    <EffectWithOptions
      title="Pause"
      help="Pauses an effect-chain for a specified duration (mostly useful in Sequences)"
      icon={<PauseCircleOutline />}
      onDelete={onDelete}
    >
      <TextField
        label="Seconds"
        type="number"
        defaultValue={millis / 1000}
        onChange={(e) => triggerChangeDebounced(+e.target.value)}
        sx={{ maxWidth: 100 }}
      />{' '}
    </EffectWithOptions>
  );
};

export default EditPauseEffect;
