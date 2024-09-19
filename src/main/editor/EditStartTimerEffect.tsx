import React, { useEffect, useId, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { Add } from '@mui/icons-material';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Checkbox from '@mui/material/Checkbox';
import FormControl from '@mui/material/FormControl';
import FormControlLabel from '@mui/material/FormControlLabel';
import Radio from '@mui/material/Radio';
import RadioGroup from '@mui/material/RadioGroup';
import TextField from '@mui/material/TextField';

import {
  $errorForID,
  forgetError,
  insertNewEffectOrTimerEffect,
  setError,
  setTimerField,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import { EffectWithID } from '../../generated/EffectWithID';
import { Timer } from '../../generated/Timer';
import { TimerStartPolicy } from '../../generated/TimerStartPolicy';
import { UUID } from '../../generated/UUID';
import { EffectVariant } from './effect-utils';
import { TimerEffectVariant } from './effect-utils';
import { AutocompleteEffectAndTimerEffect } from './widgets/AutocompleteEffect';
import ControlledTextField from './widgets/ControlledTextField';
import EditDuration from './widgets/EditDuration';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';
import EffectList from './widgets/EffectList';

const VARIANT_WITH_VALUE: TimerStartPolicy['variant'] =
  'StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching';

const EditStartTimerEffect: React.FC<{
  triggerID: UUID;
  timerSelector: TriggerEditorSelector<Timer>;
  onDelete: () => void;
}> = ({ triggerID, timerSelector, onDelete }) => {
  const dispatch = useDispatch();
  const timer = useSelector(triggerEditorSelector(timerSelector));

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

  const $$effects: TriggerEditorSelector<EffectWithID[]> = (slice) => {
    return timerSelector(slice).effects;
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
        <CreateEffectOrTimerEffectButton
          create={(variant) => {
            dispatch(
              insertNewEffectOrTimerEffect({
                variant,
                index: 0,
                triggerID: triggerID,
                seqSelector: $$effects,
              })
            );
          }}
        />
        {timer.effects.length ? (
          <EffectList triggerID={triggerID} selector={$$effects} />
        ) : (
          <p>This Timer currently has no Effects. Do you want to create one?</p>
        )}
      </CardContent>
    </Card>
  );
};

const CreateEffectOrTimerEffectButton: React.FC<{
  create: (variant: EffectVariant | TimerEffectVariant) => void;
}> = ({ create }) => {
  const [isOpen, setIsOpen] = useState(false);

  if (!isOpen) {
    return (
      <Button
        variant="contained"
        size="large"
        startIcon={<Add />}
        onClick={() => setIsOpen(true)}
        sx={{ width: 250 }}
      >
        Add New Timer Effect
      </Button>
    );
  }

  return (
    <AutocompleteEffectAndTimerEffect
      close={() => setIsOpen(false)}
      onSelect={create}
    />
  );
};

export default EditStartTimerEffect;
