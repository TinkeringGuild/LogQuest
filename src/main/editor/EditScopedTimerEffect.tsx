import {
  AlarmOnOutlined,
  HourglassBottomOutlined,
  HourglassTopOutlined,
  LabelOffOutlined,
  LabelOutlined,
  RestartAltOutlined,
  TimerOffOutlined,
  VisibilityOffOutlined,
  VisibilityOutlined,
} from '@mui/icons-material';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Paper from '@mui/material/Paper';
import TextField from '@mui/material/TextField';
import React, { ReactElement, ReactNode } from 'react';
import { useSelector, useDispatch } from 'react-redux';

import {
  editorSelector,
  EditorSelector,
  setWaitUntilFilterMatchesDuration,
  TimerEffectWaitUntilFilterMatchesType,
} from '../../features/triggers/editorSlice';
import { FilterWithContext } from '../../generated/FilterWithContext';
import { TimerEffect } from '../../generated/TimerEffect';
import { TimerTag } from '../../generated/TimerTag';
import { UUID } from '../../generated/UUID';
import EditDuration from './widgets/EditDuration';
import EditFilter from './widgets/EditFilter';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';

const TIMER_EFFECT_COMPONENTS = {
  WaitUntilFilterMatches(
    selector: EditorSelector<TimerEffectWaitUntilFilterMatchesType>,
    onDelete: () => void
  ) {
    const dispatch = useDispatch();
    const timerEffect = useSelector(editorSelector(selector));
    const [_, durationMaybe] = timerEffect.value;
    const filterSelector: EditorSelector<FilterWithContext> = (state) =>
      selector(state).value[0];
    return (
      <TimerEffectWithOptions
        title="Wait Until Filter Matches"
        help="Pause the execution of subsequent Effects until one of the given patterns matches"
        icon={<HourglassTopOutlined />}
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
        title="Wait until Seconds Remain"
        help="Pause the execution of subsequent Effects until the Timer has the specified number of seconds remaining before completion."
        icon={<HourglassBottomOutlined />}
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
        title="Add Timer Tag"
        help="Adds a Timer Tag to this Timer. A Timer Tag can be used for filtering and styling of Timers in the Overlay."
        icon={<LabelOutlined />}
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
        title="Remove Timer Tag"
        help="Removes a Timer Tag from this Timer."
        icon={<LabelOffOutlined />}
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
      <SimpleTimerEffect
        title="Clear this Timer"
        help="Kills this Timer when this Effect is ran."
        icon={<TimerOffOutlined />}
        onDelete={onDelete}
      />
    );
  },

  RestartTimer(onDelete: () => void) {
    return (
      <SimpleTimerEffect
        title="Restart this Timer"
        help="Restart this Timer with its original Duration."
        icon={<RestartAltOutlined />}
        onDelete={onDelete}
      />
    );
  },

  HideTimer(onDelete: () => void) {
    return (
      <SimpleTimerEffect
        title="Hide this Timer"
        help="Hides the Timer but keeps it running. You can use Unhide Timer to show it again."
        icon={<VisibilityOffOutlined />}
        onDelete={onDelete}
      />
    );
  },

  UnhideTimer(onDelete: () => void) {
    return (
      <SimpleTimerEffect
        title="Un-Hide this Timer"
        help="Shows a previously hidden Timer"
        icon={<VisibilityOutlined />}
        onDelete={onDelete}
      />
    );
  },

  WaitUntilFinished(onDelete: () => void) {
    return (
      <SimpleTimerEffect
        title="Wait until Finished"
        help="Waits until the Timer has run its course. This does not execute if the Timer has been killed. You probably want to use this in a Sequence"
        icon={<AlarmOnOutlined />}
        onDelete={onDelete}
      />
    );
  },
};

const EditScopedTimerEffect: React.FC<{
  triggerID: UUID;
  timerSelector: EditorSelector<TimerEffect>;
  onDelete: () => void;
}> = ({ triggerID: _, timerSelector, onDelete }) => {
  const timerEffect = useSelector(editorSelector(timerSelector));
  const variant = timerEffect.variant;
  if (
    variant === 'IncrementCounter' ||
    variant === 'DecrementCounter' ||
    variant === 'ResetCounter'
  ) {
    return <p>TODO</p>;
  } else if (variant === 'WaitUntilFilterMatches') {
    return TIMER_EFFECT_COMPONENTS[variant](
      timerSelector as EditorSelector<TimerEffectWaitUntilFilterMatchesType>,
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

const SimpleTimerEffect: React.FC<{
  title: string;
  help: string;
  icon: ReactElement;
  onDelete: () => void;
}> = ({ title, help, icon, onDelete }) => (
  <EffectHeader onDelete={onDelete}>
    <Paper elevation={10} sx={{ margin: '0 auto', padding: '10px 15px' }}>
      <EffectTitle title={title} help={help} icon={icon} />
    </Paper>
  </EffectHeader>
);

const TimerEffectWithOptions: React.FC<{
  title: string;
  help: string;
  icon: ReactElement;
  children: ReactNode;
  onDelete: () => void;
}> = ({ title, help, icon, children, onDelete }) => {
  return (
    <Card elevation={10}>
      <CardHeader
        title={
          <EffectHeader onDelete={onDelete}>
            <EffectTitle title={title} help={help} icon={icon} />
          </EffectHeader>
        }
      />
      <CardContent>{children}</CardContent>
    </Card>
  );
};

export default EditScopedTimerEffect;
