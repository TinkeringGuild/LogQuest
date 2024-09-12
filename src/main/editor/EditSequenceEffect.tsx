import {
  ArrowDownward,
  KeyboardDoubleArrowDownOutlined,
} from '@mui/icons-material';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Divider from '@mui/material/Divider';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import { useSelector, useDispatch } from 'react-redux';

import {
  deleteEffect,
  TriggerEditorSelector,
  triggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import { EffectWithID } from '../../generated/EffectWithID';
import { UUID } from '../../generated/UUID';
import EditEffect from './EditEffect';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';

const EditSequenceEffect: React.FC<{
  triggerID: UUID;
  seqSelector: TriggerEditorSelector<EffectWithID[]>;
  onDelete: () => void;
}> = ({ triggerID, seqSelector, onDelete }) => {
  const dispatch = useDispatch();
  const seq = useSelector(triggerEditorSelector(seqSelector));

  const helpText =
    'Sequences execute effects in-order, waiting until each is done before continuing.';
  return (
    <Card elevation={10}>
      <CardHeader
        title={
          <EffectHeader onDelete={onDelete}>
            <EffectTitle
              title="Sequence"
              help={helpText}
              icon={<KeyboardDoubleArrowDownOutlined />}
            />
          </EffectHeader>
        }
      />
      <CardContent>
        <Stack
          direction="column"
          divider={
            <Divider sx={{ marginTop: 1, marginBottom: 1 }}>
              <Tooltip arrow followCursor placement="top" title={helpText}>
                <ArrowDownward />
              </Tooltip>
            </Divider>
          }
        >
          {seq.map((effect, index) => (
            <EditEffect
              key={effect.id}
              triggerID={triggerID}
              effectSelector={(slice) => seqSelector(slice)[index]}
              onDelete={() =>
                dispatch(
                  deleteEffect({ effectID: effect.id, selector: seqSelector })
                )
              }
            />
          ))}
        </Stack>
      </CardContent>
    </Card>
  );
};

export default EditSequenceEffect;
