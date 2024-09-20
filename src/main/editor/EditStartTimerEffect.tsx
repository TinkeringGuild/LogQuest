import React, { useEffect, useState } from 'react';
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
  insertNewEffectOrTimerEffect,
  setTimerField,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import { EffectWithID } from '../../generated/EffectWithID';
import { Timer } from '../../generated/Timer';
import { TimerStartPolicy } from '../../generated/TimerStartPolicy';
import { UUID } from '../../generated/UUID';
import { EffectVariant, TimerEffectVariant } from './effect-utils';
import { createEffectOrTimerEffectAutocomplete } from './widgets/AutocompleteEffect';
import ControlledTextField from './widgets/ControlledTextField';
import EditDuration from './widgets/EditDuration';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';
import EffectList from './widgets/EffectList';
import IncludeTimerEffectsContext from './widgets/IncludeTimerEffectsContext';

const VARIANT_WITH_VALUE: TimerStartPolicy['variant'] =
  'StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching';

const START_POLICY_INPUT_FIELD_ERROR_ID =
  'trigger-editor-start-policy-input-field';

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

  const startPolicyErrorMessage = useSelector(
    $errorForID(START_POLICY_INPUT_FIELD_ERROR_ID)
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

  const $$effects: TriggerEditorSelector<EffectWithID[]> = (slice) => {
    return timerSelector(slice).effects;
  };

  const insertNewVariantAtIndex = (
    variant: EffectVariant | TimerEffectVariant,
    index: number
  ) => {
    dispatch(
      insertNewEffectOrTimerEffect({
        variant,
        index,
        triggerID: triggerID,
        seqSelector: $$effects,
      })
    );
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
            fullWidth
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
              value={startPolicyVariant}
              onChange={(e) => {
                const variant = e.target.value as TimerStartPolicy['variant'];
                setStartPolicyVariant(variant);
                if (
                  variant !== VARIANT_WITH_VALUE &&
                  startPolicyValue !== null
                ) {
                  setStartPolicyValue(null);
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
                className="template-input"
                onCommit={(input) => setStartPolicyValue(input)}
                validate={(value) =>
                  value.trim()
                    ? null
                    : 'You must specify a Timer name to replace'
                }
                errorID={START_POLICY_INPUT_FIELD_ERROR_ID}
                sx={{ ml: 3.75, mt: 0.5, width: 480 }}
              />
            </div>
          )}
        </Box>

        <h3>Timer Effects</h3>
        <CreateEffectOrTimerEffectButton
          create={(variant) => insertNewVariantAtIndex(variant, 0)}
        />
        {timer.effects.length ? (
          <IncludeTimerEffectsContext.Provider value={true}>
            <EffectList triggerID={triggerID} selector={$$effects} />
          </IncludeTimerEffectsContext.Provider>
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
        sx={{ width: 275 }}
      >
        Add New Timer Effect
      </Button>
    );
  }

  return createEffectOrTimerEffectAutocomplete({
    onSelect: create,
    close: () => setIsOpen(false),
  });
};

export default EditStartTimerEffect;
