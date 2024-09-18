import { debounce } from 'lodash';
import { useDispatch, useSelector } from 'react-redux';

import { PauseCircleOutline } from '@mui/icons-material';

import {
  EffectVariantPause,
  setPauseDuration,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import EffectWithOptions from './EffectWithOptions';
import EditDuration from './widgets/EditDuration';

const DEBOUNCE_WAIT_MILLIS = 300;

const EditPauseEffect: React.FC<{
  selector: TriggerEditorSelector<EffectVariantPause>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const dispatch = useDispatch();
  const { value: millis } = useSelector(triggerEditorSelector(selector));

  const triggerChange = (seconds: number) => {
    dispatch(setPauseDuration({ millis: seconds * 1000, selector }));
  };

  const triggerChangeDebounced = debounce(triggerChange, DEBOUNCE_WAIT_MILLIS);

  return (
    <EffectWithOptions
      title="Pause"
      help="Pauses an effect-chain for a specified duration (mostly useful in Sequences)"
      icon={<PauseCircleOutline />}
      width={300}
      onDelete={onDelete}
    >
      <EditDuration
        millis={millis}
        onChange={(value) => triggerChangeDebounced(value)}
      />
    </EffectWithOptions>
  );
};

export default EditPauseEffect;
