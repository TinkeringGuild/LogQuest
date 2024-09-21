import TextField from '@mui/material/TextField';
import { HTMLProps, useEffect, useMemo, useState } from 'react';

import { Duration } from '../../../generated/Duration';
import EditorError from './EditorError';

const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;

const floor = Math.floor;

const EditDuration: React.FC<{
  millis: number;
  onChange: (value: Duration) => void;
  errorProps?: HTMLProps<HTMLDivElement>;
}> = ({ millis, onChange, errorProps }) => {
  const [durationValues, setDurationValues] = useState({
    seconds: 0,
    minutes: 0,
    hours: 0,
    totalMillis: 0,
  });

  // Initialize and synchronize the component state with the prop
  useEffect(() => {
    setDurationValues({
      ...destructureDuration(millis),
      totalMillis: millis,
    });
  }, [millis]);

  // Invoke onChange callback anytime the duration changes
  useEffect(() => {
    if (durationValues.totalMillis > 0) {
      onChange(durationValues.totalMillis);
    }
  }, [durationValues.totalMillis]);

  // Memoize the errorMessage based on the duration changing
  const errorMessage: string | undefined = useMemo(() => {
    if (durationValues.totalMillis === 0) {
      return 'The duration cannot be zero';
    }
    if (durationValues.totalMillis < 0) {
      return 'The duration cannot be negative';
    }
  }, [durationValues.totalMillis]);

  // State-setter for onChange callbacks
  const updateDuration = (
    update: { seconds: number } | { minutes: number } | { hours: number }
  ) => {
    const { seconds, minutes, hours } = { ...durationValues, ...update };
    if (seconds < 0 || minutes < 0 || hours < 0) {
      return;
    }
    const totalMillis = calculateDuration(seconds, minutes, hours);
    setDurationValues({
      seconds,
      minutes,
      hours,
      totalMillis,
    });
  };

  return (
    <>
      <div>
        <TextField
          label="Hours"
          size="small"
          type="number"
          error={!!errorMessage}
          value={durationValues.hours}
          onChange={(e) => updateDuration({ hours: +e.target.value })}
          sx={{ maxWidth: 80 }}
        />{' '}
        <TextField
          label="Minutes"
          size="small"
          type="number"
          error={!!errorMessage}
          value={durationValues.minutes}
          onChange={(e) => updateDuration({ minutes: +e.target.value })}
          sx={{ maxWidth: 80 }}
        />{' '}
        <TextField
          label="Seconds"
          size="small"
          type="number"
          error={!!errorMessage}
          value={durationValues.seconds}
          onChange={(e) => updateDuration({ seconds: +e.target.value })}
          sx={{ maxWidth: 100 }}
        />
      </div>
      {errorMessage && (
        <div {...{ style: { marginTop: 10 }, ...errorProps }}>
          <EditorError center width={237} message={errorMessage} />
        </div>
      )}
    </>
  );
};

function calculateDuration(
  seconds: number,
  minutes: number,
  hours: number
): Duration {
  return hours * HOUR + minutes * MINUTE + seconds * SECOND;
}

function destructureDuration(millis: Duration): {
  seconds: number;
  minutes: number;
  hours: number;
} {
  const hours = floor(millis / HOUR);
  let remainder = millis % HOUR;

  const minutes = floor(remainder / MINUTE);
  remainder %= MINUTE;

  const seconds = floor(remainder / SECOND);

  return {
    seconds,
    minutes,
    hours,
  };
}

export default EditDuration;
