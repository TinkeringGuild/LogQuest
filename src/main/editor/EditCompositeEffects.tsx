import React, { ReactElement } from 'react';
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
import Typography from '@mui/material/Typography';
import { EffectVariant } from './effect-utils';

export const EditSequenceEffect: React.FC<{
  triggerID: UUID;
  seqSelector: TriggerEditorSelector<EffectWithID[]>;
  onDelete: () => void;
}> = (props) => (
  <EditCompositeEffect
    variant="Sequence"
    help="Sequences execute effects in-order, waiting until each is done before continuing."
    seqIcon={<ArrowDownward sx={{ color: 'black' }} />}
    {...props}
  />
);

export const EditParallelEffect: React.FC<{
  triggerID: UUID;
  seqSelector: TriggerEditorSelector<EffectWithID[]>;
  onDelete: () => void;
}> = (props) => (
  <EditCompositeEffect
    variant="Parallel"
    help="Executes effects in Parallel. You can embed Sequences inside this Parallel to run multiple sequences concurrently."
    seqIcon={<AmpersandIcon />}
    {...props}
  />
);

const EditCompositeEffect: React.FC<{
  variant: EffectVariant;
  help: string;
  triggerID: UUID;
  seqSelector: TriggerEditorSelector<EffectWithID[]>;
  seqIcon: ReactElement;
  onDelete: () => void;
}> = ({ variant, help, triggerID, seqSelector, seqIcon, onDelete }) => {
  const dispatch = useDispatch();
  const seq = useSelector(triggerEditorSelector(seqSelector));

  const insertEffectAtIndex: (
    insertedVariant: Effect['variant'],
    index: number
  ) => void = (insertedVariant, index) => {
    dispatch(
      insertEffect({ variant: insertedVariant, index, triggerID, seqSelector })
    );
  };

  return (
    <Card elevation={10}>
      <CardHeader
        title={
          <EffectHeader onDelete={onDelete}>
            <EffectTitle variant={variant} help={help} />
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
                defaultIcon={index === seq.length - 1 ? <Add /> : seqIcon}
              />
            </React.Fragment>
          ))}
        </Stack>
      </CardContent>
    </Card>
  );
};

const AmpersandIcon: React.FC<{}> = () => (
  <Typography color="black" fontSize={20} fontWeight="bold" mt={-0.3}>
    &amp;
  </Typography>
);
