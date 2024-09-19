import { useDispatch, useSelector } from 'react-redux';

import Stack from '@mui/material/Stack';

import {
  deleteEffect,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../../features/triggers/triggerEditorSlice';
import { EffectWithID } from '../../../generated/EffectWithID';
import { UUID } from '../../../generated/UUID';
import EditEffect from '../EditEffect';

const EffectList: React.FC<{
  triggerID: UUID;
  selector: TriggerEditorSelector<EffectWithID[]>;
}> = ({ triggerID, selector }) => {
  const dispatch = useDispatch();
  const effects = useSelector(triggerEditorSelector(selector));
  return (
    <Stack gap={2}>
      {effects.map((effect) => {
        return (
          <EditEffect
            key={effect.id}
            triggerID={triggerID}
            onDelete={() =>
              dispatch(
                deleteEffect({
                  effectID: effect.id,
                  selector,
                })
              )
            }
            effectSelector={(slice) => {
              const effects = selector(slice);
              return effects.find(({ id }) => id === effect.id)!;
            }}
          />
        );
      })}
    </Stack>
  );
};

export default EffectList;
