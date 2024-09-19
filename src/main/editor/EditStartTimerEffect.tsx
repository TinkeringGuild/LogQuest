import { useEffect, useId, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Checkbox from '@mui/material/Checkbox';
import FormControl from '@mui/material/FormControl';
import FormControlLabel from '@mui/material/FormControlLabel';
import Paper from '@mui/material/Paper';
import Radio from '@mui/material/Radio';
import RadioGroup from '@mui/material/RadioGroup';
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';

import {
  $errorForID,
  deleteEffect,
  forgetError,
  setError,
  setTimerField,
  triggerEditorSelector,
  TriggerEditorSelector,
  TriggerEditorState,
} from '../../features/triggers/triggerEditorSlice';
import { Timer } from '../../generated/Timer';
import { TimerStartPolicy } from '../../generated/TimerStartPolicy';
import EditEffect from './EditEffect';
import ControlledTextField from './widgets/ControlledTextField';
import EditDuration from './widgets/EditDuration';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';

const VARIANT_WITH_VALUE: TimerStartPolicy['variant'] =
  'StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching';

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

  const [startPolicyVariant, setStartPolicyVariant] = useState<
    TimerStartPolicy['variant']
  >(timer.start_policy.variant);

  const [startPolicyValue, setStartPolicyValue] = useState(
    timer.start_policy.variant === VARIANT_WITH_VALUE
      ? timer.start_policy.value
      : null
  );

  const startPolicyValueFieldID = useId();

  const startPolicyErrorMessage = useSelector(
    $errorForID(startPolicyValueFieldID)
  );

  // Updates the Timer in the store with its TimerStartPolicy value
  useEffect(() => {
    if (startPolicyErrorMessage) {
      return;
    }

    const startPolicy: TimerStartPolicy =
      startPolicyVariant === VARIANT_WITH_VALUE
        ? { variant: startPolicyVariant, value: startPolicyValue! }
        : { variant: startPolicyVariant };

    dispatch(
      setTimerField({
        field: 'start_policy',
        value: startPolicy,
        selector: timerSelector,
      })
    );
  }, [startPolicyVariant, startPolicyValue]);

  // Cleans up the error message from the store on un-mount
  useEffect(() => {
    return () => {
      dispatch(forgetError(startPolicyValueFieldID));
    };
  }, []);

  const setStartPolicyError = (error: string | null) => {
    const action = error
      ? setError({ id: startPolicyValueFieldID, error })
      : forgetError(startPolicyValueFieldID);
    dispatch(action);
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
            sx={{ width: 400 }}
            label="Timer Name (Template)"
            defaultValue={timer.name_tmpl}
            className="template-input"
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
          <h3>Timer Duration</h3>
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
        <Box mt={2}>
          <h3 style={{ marginBottom: 5 }}>Timer Start Behavior</h3>
          <FormControl>
            <RadioGroup
              defaultValue={startPolicyVariant}
              onChange={(e) => {
                const variant = e.target.value as TimerStartPolicy['variant'];
                setStartPolicyVariant(variant);
                if (
                  variant !== VARIANT_WITH_VALUE &&
                  startPolicyValue !== null
                ) {
                  setStartPolicyValue(null);
                  forgetError(startPolicyValueFieldID);
                }
              }}
            >
              <FormControlLabel
                value="AlwaysStartNewTimer"
                control={<Radio />}
                label="Always start new Timer"
              />
              <FormControlLabel
                value="DoNothingIfTimerRunning"
                control={<Radio />}
                label="Do nothing if Timer is already running"
              />
              <FormControlLabel
                value="StartAndReplacesAllTimersOfTrigger"
                control={<Radio />}
                label="Start and replace all other Timers of this Trigger"
              />
              <FormControlLabel
                value="StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching"
                control={<Radio />}
                label="Start and replace any Timer (of this Trigger) with a specific name..."
              />
            </RadioGroup>
          </FormControl>
          {startPolicyVariant === VARIANT_WITH_VALUE && (
            <div>
              <ControlledTextField
                label="Name of Timer(s) to replace (Template)"
                value={startPolicyValue || ''}
                error={!!startPolicyErrorMessage}
                helperText={startPolicyErrorMessage}
                id={startPolicyValueFieldID}
                className="template-input"
                onCommit={(input) => setStartPolicyValue(input)}
                validate={(value) =>
                  value.trim()
                    ? null
                    : 'You must specify a Timer name to replace'
                }
                onValidateChange={setStartPolicyError}
                sx={{ ml: 3.75, mt: 0.5, width: 480 }}
              />
            </div>
          )}
        </Box>
        <h3>Timer Effects</h3>
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
