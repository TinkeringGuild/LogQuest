import React, { ReactNode } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import TextField from '@mui/material/TextField';

import {
  setWaitUntilFilterMatchesDuration,
  TimerEffectWaitUntilFilterMatchesType,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import { FilterWithContext } from '../../generated/FilterWithContext';
import { TimerEffect } from '../../generated/TimerEffect';
import { TimerTag } from '../../generated/TimerTag';
import { UUID } from '../../generated/UUID';
import {
  HUMANIZED_TIMER_EFFECT_NAMES,
  TimerEffectIcon,
  TimerEffectVariant,
} from './effect-utils';
import EffectWithoutOptions from './EffectWithoutOptions';
import EditDuration from './widgets/EditDuration';
import EditFilter from './widgets/EditFilter';
import { EffectHeader, ExplicitEffectTitle } from './widgets/EffectHeader';

const TIMER_EFFECT_COMPONENTS = {
  WaitUntilFilterMatches(
    selector: TriggerEditorSelector<TimerEffectWaitUntilFilterMatchesType>,
    onDelete: () => void
  ) {
    const dispatch = useDispatch();
    const timerEffect = useSelector(triggerEditorSelector(selector));
    const [_, durationMaybe] = timerEffect.value;
    const filterSelector: TriggerEditorSelector<FilterWithContext> = (state) =>
      selector(state).value[0];
    return (
      <TimerEffectWithOptions
        variant="WaitUntilFilterMatches"
        help="Pause the execution of subsequent Effects until one of the given patterns matches"
        onDelete={onDelete}
      >
        <EditFilter selector={filterSelector} />
        {/* TODO: There should be a checkbox to enable timeout, and EditDuration should validate it's not zero */}
        <h5>Timeout (Optional)</h5>
        <EditDuration
          millis={durationMaybe || 0}
          onChange={(duration) =>
            dispatch(
              setWaitUntilFilterMatchesDuration({
                duration,
                selector,
              })
            )
          }
        />
      </TimerEffectWithOptions>
    );
  },
  WaitUntilSecondsRemain(seconds: number, onDelete: () => void) {
    return (
      <TimerEffectWithOptions
        variant="WaitUntilSecondsRemain"
        help="Pause the execution of subsequent Effects until the Timer has the specified number of seconds remaining before completion."
        onDelete={onDelete}
      >
        <TextField
          label="Seconds"
          type="number"
          variant="outlined"
          defaultValue={seconds}
          sx={{ maxWidth: 80 }}
        />
      </TimerEffectWithOptions>
    );
  },

  AddTag(timerTag: TimerTag, onDelete: () => void) {
    return (
      <TimerEffectWithOptions
        variant="AddTag"
        help="Adds a Timer Tag to this Timer. A Timer Tag can be used for filtering and styling of Timers in the Overlay."
        onDelete={onDelete}
      >
        <TextField
          label="Timer Tag"
          variant="outlined"
          defaultValue={timerTag}
          sx={{ maxWidth: 200 }}
        />
      </TimerEffectWithOptions>
    );
  },

  RemoveTag(timerTag: TimerTag, onDelete: () => void) {
    return (
      <TimerEffectWithOptions
        variant="RemoveTag"
        help="Removes a Timer Tag from this Timer."
        onDelete={onDelete}
      >
        <TextField
          label="Timer Tag"
          variant="outlined"
          defaultValue={timerTag}
          sx={{ maxWidth: 200 }}
        />
      </TimerEffectWithOptions>
    );
  },

  ClearTimer(onDelete: () => void) {
    return (
      <EffectWithoutOptions
        variant="ClearTimer"
        help="Kills this Timer when this Effect is ran."
        onDelete={onDelete}
      />
    );
  },

  RestartTimer(onDelete: () => void) {
    return (
      <EffectWithoutOptions
        variant="RestartTimer"
        help="Restart this Timer with its original Duration."
        onDelete={onDelete}
      />
    );
  },

  HideTimer(onDelete: () => void) {
    return (
      <EffectWithoutOptions
        variant="HideTimer"
        help="Hides the Timer but keeps it running. You can use Unhide Timer to show it again."
        onDelete={onDelete}
      />
    );
  },

  UnhideTimer(onDelete: () => void) {
    return (
      <EffectWithoutOptions
        variant="UnhideTimer"
        help="Shows a previously hidden Timer"
        onDelete={onDelete}
      />
    );
  },

  WaitUntilFinished(onDelete: () => void) {
    return (
      <EffectWithoutOptions
        variant="WaitUntilFinished"
        help="Waits until the Timer has run its course. This does not execute if the Timer has been killed. You probably want to use this in a Sequence"
        onDelete={onDelete}
      />
    );
  },
};

const EditScopedTimerEffect: React.FC<{
  triggerID: UUID;
  timerSelector: TriggerEditorSelector<TimerEffect>;
  onDelete: () => void;
}> = ({ triggerID: _, timerSelector, onDelete }) => {
  const timerEffect = useSelector(triggerEditorSelector(timerSelector));
  const variant = timerEffect.variant;
  if (
    variant === 'IncrementCounter' ||
    variant === 'DecrementCounter' ||
    variant === 'ResetCounter'
  ) {
    return <p>TODO</p>;
  } else if (variant === 'WaitUntilFilterMatches') {
    return TIMER_EFFECT_COMPONENTS[variant](
      timerSelector as TriggerEditorSelector<TimerEffectWaitUntilFilterMatchesType>,
      onDelete
    );
  } else if (variant === 'WaitUntilSecondsRemain') {
    return TIMER_EFFECT_COMPONENTS[variant](timerEffect.value, onDelete);
  } else if (variant === 'AddTag') {
    return TIMER_EFFECT_COMPONENTS[variant](timerEffect.value, onDelete);
  } else if (variant === 'RemoveTag') {
    return TIMER_EFFECT_COMPONENTS[variant](timerEffect.value, onDelete);
  } else {
    return TIMER_EFFECT_COMPONENTS[timerEffect.variant](onDelete);
  }
};

const TimerEffectWithOptions: React.FC<{
  variant: TimerEffectVariant;
  help: string;
  children: ReactNode;
  onDelete: () => void;
}> = ({ variant, help, children, onDelete }) => {
  const title = HUMANIZED_TIMER_EFFECT_NAMES[variant];
  const VariantIcon = TimerEffectIcon[variant];
  return (
    <Card elevation={10}>
      <CardHeader
        title={
          <EffectHeader onDelete={onDelete}>
            <ExplicitEffectTitle
              title={title}
              help={help}
              icon={<VariantIcon />}
            />
          </EffectHeader>
        }
      />
      <CardContent>{children}</CardContent>
    </Card>
  );
};

export default EditScopedTimerEffect;
