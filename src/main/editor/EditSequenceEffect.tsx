import React from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { Add } from '@mui/icons-material';
import ArrowDownward from '@mui/icons-material/ArrowDownward';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Stack from '@mui/material/Stack';

import {
  deleteEffect,
  insertEffect,
  TriggerEditorSelector,
  triggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import { Effect } from '../../generated/Effect';
import { EffectWithID } from '../../generated/EffectWithID';
import { UUID } from '../../generated/UUID';
import EditEffect from './EditEffect';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';
import InsertEffectDivider from './widgets/InsertEffectDivider';

const HELP_TEXT =
  'Sequences execute effects in-order, waiting until each is done before continuing.';

const EditSequenceEffect: React.FC<{
  triggerID: UUID;
  seqSelector: TriggerEditorSelector<EffectWithID[]>;
  onDelete: () => void;
}> = ({ triggerID, seqSelector, onDelete }) => {
  const dispatch = useDispatch();
  const seq = useSelector(triggerEditorSelector(seqSelector));

  const insertEffectAtIndex: (
    variant: Effect['variant'],
    index: number
  ) => void = (variant, index) => {
    dispatch(insertEffect({ variant, index, triggerID, seqSelector }));
  };

  return (
    <Card elevation={10}>
      <CardHeader
        title={
          <EffectHeader onDelete={onDelete}>
            <EffectTitle variant="Sequence" help={HELP_TEXT} />
          </EffectHeader>
        }
        sx={{ pb: 0 }}
      />
      <CardContent sx={{ pt: 0 }}>
        <InsertEffectDivider
          index={0}
          onInsertEffect={insertEffectAtIndex}
          defaultIcon={<Add />}
        />
        <Stack direction="column">
          {seq.map((effect, index) => (
            <React.Fragment key={effect.id}>
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
              <InsertEffectDivider
                index={index + 1}
                onInsertEffect={insertEffectAtIndex}
                defaultIcon={
                  index === seq.length - 1 ? (
                    <Add />
                  ) : (
                    <ArrowDownward sx={{ color: 'black' }} />
                  )
                }
              />
            </React.Fragment>
          ))}
        </Stack>
      </CardContent>
    </Card>
  );
};

export default EditSequenceEffect;
