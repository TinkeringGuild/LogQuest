import React, { ReactElement, useContext } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { Add } from '@mui/icons-material';
import ArrowDownward from '@mui/icons-material/ArrowDownward';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import {
  deleteEffect,
  insertNewEffect,
  insertNewEffectOrTimerEffect,
  TriggerEditorSelector,
  triggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import { EffectWithID } from '../../generated/EffectWithID';
import { UUID } from '../../generated/UUID';
import EditEffect from './EditEffect';
import { EffectVariant, TimerEffectVariant } from './effect-utils';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';
import IncludeTimerEffectsContext from './widgets/IncludeTimerEffectsContext';
import InsertEffectDivider from './widgets/InsertEffectDivider';

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

  const includeTimerEffects = useContext(IncludeTimerEffectsContext);

  const insertEffectAtIndex: (
    insertedVariant: string,
    index: number
  ) => void = (insertedVariant, index) => {
    const actionParams = {
      index,
      triggerID,
      seqSelector,
    };
    const action = includeTimerEffects
      ? insertNewEffectOrTimerEffect({
          variant: insertedVariant as EffectVariant | TimerEffectVariant,
          ...actionParams,
        })
      : insertNewEffect({
          variant: insertedVariant as EffectVariant,
          ...actionParams,
        });

    dispatch(action);
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
          defaultIcon={<Add />}
          onInsertEffect={(variant) => insertEffectAtIndex(variant, 0)}
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
                    deleteEffect({
                      effectID: effect.id,
                      selector: seqSelector,
                    })
                  )
                }
              />
              <InsertEffectDivider
                index={index + 1}
                defaultIcon={index === seq.length - 1 ? <Add /> : seqIcon}
                onInsertEffect={(variant) =>
                  insertEffectAtIndex(variant, index + 1)
                }
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
