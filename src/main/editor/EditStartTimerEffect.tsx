import { useDispatch, useSelector } from 'react-redux';

import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Checkbox from '@mui/material/Checkbox';
import FormControlLabel from '@mui/material/FormControlLabel';
import Paper from '@mui/material/Paper';
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';

import {
  deleteEffect,
  setTimerField,
  triggerEditorSelector,
  TriggerEditorSelector,
  TriggerEditorState,
} from '../../features/triggers/triggerEditorSlice';
import { Timer } from '../../generated/Timer';
import EditEffect from './EditEffect';
import EditDuration from './widgets/EditDuration';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';

const EditStartTimerEffect: React.FC<{
  timerSelector: TriggerEditorSelector<Timer>;
  onDelete: () => void;
}> = ({ timerSelector, onDelete }) => {
  const dispatch = useDispatch();
  const timer = useSelector(triggerEditorSelector(timerSelector));

  const $effects = (state: TriggerEditorState) => {
    const timer: Timer = timerSelector(state);
    return timer.effects;
  };

  return (
    <Card elevation={10}>
      <CardHeader
        title={
          <EffectHeader onDelete={onDelete}>
            <EffectTitle
              variant="StartTimer"
              help="Immediately create a new Timer with the given parameters"
            />
          </EffectHeader>
        }
      />
      <CardContent>
        <div>
          <TextField
            sx={{ minWidth: 300 }}
            label="Timer Name (Template)"
            defaultValue={timer.name_tmpl}
            onBlur={(e) =>
              dispatch(
                setTimerField({
                  field: 'name_tmpl',
                  value: e.target.value,
                  selector: timerSelector,
                })
              )
            }
          />
        </div>
        <div>
          <h4>Timer Duration</h4>
          <EditDuration
            millis={timer.duration}
            onChange={(duration) => {
              dispatch(
                setTimerField({
                  field: 'duration',
                  value: duration,
                  selector: timerSelector,
                })
              );
            }}
          />
        </div>
        <FormControlLabel
          label="Timer repeats when finished"
          control={
            <Checkbox
              checked={timer.repeats}
              onChange={(e) =>
                dispatch(
                  setTimerField({
                    field: 'repeats',
                    value: e.target.checked,
                    selector: timerSelector,
                  })
                )
              }
            />
          }
        />
        {/* TODO: timer.tags */}
        <p>
          Start policy (TODO): <code>{JSON.stringify(timer.start_policy)}</code>
        </p>
        <h4>Timer Effects:</h4>
        <Stack gap={2}>
          {timer.effects.map((effect, index) => (
            <Paper key={effect.id}>
              <EditEffect
                triggerID={timer.trigger_id}
                effectSelector={(slice) => timerSelector(slice).effects[index]}
                onDelete={() =>
                  dispatch(
                    deleteEffect({ effectID: effect.id, selector: $effects })
                  )
                }
              />
            </Paper>
          ))}
        </Stack>
      </CardContent>
    </Card>
  );
};

export default EditStartTimerEffect;
