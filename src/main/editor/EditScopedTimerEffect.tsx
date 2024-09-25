import React, { useEffect, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import Box from '@mui/material/Box';
import Checkbox from '@mui/material/Checkbox';
import FormControlLabel from '@mui/material/FormControlLabel';
import TextField from '@mui/material/TextField';

import {
  setWaitUntilFilterMatchesDuration,
  setWaitUntilSecondsRemainSeconds,
  TimerEffectWaitUntilFilterMatchesType,
  TimerEffectWaitUntilSecondsRemainType,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import { Duration } from '../../generated/Duration';
import { FilterWithContext } from '../../generated/FilterWithContext';
import { TimerEffect } from '../../generated/TimerEffect';
import { TimerTag } from '../../generated/TimerTag';
import { UUID } from '../../generated/UUID';
import EffectWithOptions from './EffectWithOptions';
import EffectWithoutOptions from './EffectWithoutOptions';
import EditDuration from './widgets/EditDuration';
import EditFilter from './widgets/EditFilter';
import EditorError from './widgets/EditorError';

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

    const setDuration = (duration: Duration | null) =>
      dispatch(
        setWaitUntilFilterMatchesDuration({
          duration,
          selector,
        })
      );

    return (
      <EffectWithOptions
        variant="WaitUntilFilterMatches"
        help="Pause the execution of subsequent Effects until one of the given patterns matches"
        onDelete={onDelete}
      >
        <EditFilter matchersIncludeContext={true} selector={filterSelector}>
          <Box margin="0 auto">
            <EditorError
              center
              message="This filter must have at least one valid pattern."
            />
          </Box>
        </EditFilter>
        {/* TODO: There should be a checkbox to enable timeout, and EditDuration should validate it's not zero */}
        <Box
          textAlign="center"
          justifyContent="center"
          display="flex"
          flexDirection="column"
        >
          <FormControlLabel
            label="Stop waiting after a specific duration..."
            control={<Checkbox />}
            value={durationMaybe !== null}
            onChange={(_, checked) => setDuration(checked ? 0 : null)}
            sx={{ margin: '15px auto 0' }}
          />
          {durationMaybe !== null && (
            <>
              <h4 style={{ marginTop: 0 }}>Stop Waiting Timeout</h4>
              <EditDuration
                millis={durationMaybe || 0}
                onChange={setDuration}
                errorProps={{
                  style: {
                    marginTop: 10,
                    width: 287,
                    alignSelf: 'center',
                    display: 'flex',
                    justifyContent: 'center',
                  },
                }}
              />
            </>
          )}
        </Box>
      </EffectWithOptions>
    );
  },
  WaitUntilSecondsRemain(
    seconds: number,
    selector: TriggerEditorSelector<TimerEffectWaitUntilSecondsRemainType>,
    onDelete: () => void
  ) {
    const dispatch = useDispatch();
    const [secondsInput, setSecondsInput] = useState(0);

    useEffect(() => {
      seconds !== secondsInput && setSecondsInput(seconds);
    }, [seconds]);

    return (
      <EffectWithOptions
        variant="WaitUntilSecondsRemain"
        help="Pause the execution of subsequent Effects until the Timer has the specified number of seconds remaining before completion."
        onDelete={onDelete}
        width={400}
      >
        <Box textAlign="center">
          <TextField
            label="Seconds"
            type="number"
            variant="outlined"
            value={secondsInput}
            onChange={(e) => setSecondsInput(+e.target.value)}
            onBlur={(e) => {
              const value = +e.target.value;
              dispatch(
                setWaitUntilSecondsRemainSeconds({ seconds: value, selector })
              );
            }}
            sx={{ maxWidth: 110 }}
          />
        </Box>
      </EffectWithOptions>
    );
  },

  AddTag(timerTag: TimerTag, onDelete: () => void) {
    return (
      <EffectWithOptions
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
      </EffectWithOptions>
    );
  },

  RemoveTag(timerTag: TimerTag, onDelete: () => void) {
    return (
      <EffectWithOptions
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
      </EffectWithOptions>
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
    return TIMER_EFFECT_COMPONENTS[variant](
      timerEffect.value,
      timerSelector as TriggerEditorSelector<TimerEffectWaitUntilSecondsRemainType>,
      onDelete
    );
  } else if (variant === 'AddTag') {
    return TIMER_EFFECT_COMPONENTS[variant](timerEffect.value, onDelete);
  } else if (variant === 'RemoveTag') {
    return TIMER_EFFECT_COMPONENTS[variant](timerEffect.value, onDelete);
  } else {
    return TIMER_EFFECT_COMPONENTS[timerEffect.variant](onDelete);
  }
};

export default EditScopedTimerEffect;
