import TextField from '@mui/material/TextField';
import { debounce } from 'lodash';
import { useRef } from 'react';

import { Duration } from '../../../generated/Duration';

const HOUR = 60 * 60 * 1000;
const MINUTE = 60 * 1000;
const SECOND = 1000;

const floor = Math.floor;

const EditDuration: React.FC<{
  millis: number;
  onChange: (value: Duration) => void;
}> = ({ millis, onChange }) => {
  const hoursField = useRef<HTMLInputElement>(null);
  const minutesField = useRef<HTMLInputElement>(null);
  const secondsField = useRef<HTMLInputElement>(null);

  const hours = floor(millis / HOUR);
  let remainder = millis % HOUR;

  const minutes = floor(remainder / MINUTE);
  remainder %= MINUTE;

  const seconds = floor(remainder / SECOND);

  const triggerChange = () => {
    if (hoursField.current && minutesField.current && secondsField.current) {
      const hours = +hoursField.current.value;
      const minutes = +minutesField.current.value;
      const seconds = +secondsField.current.value;
      const duration: Duration =
        hours * HOUR + minutes * MINUTE + seconds * SECOND;
      onChange(duration);
    }
  };

  const triggerChangeDebounced = debounce(triggerChange, 300);

  return (
    <div>
      <TextField
        label="Hours"
        size="small"
        type="number"
        defaultValue={hours}
        inputRef={hoursField}
        onChange={triggerChangeDebounced}
        sx={{ maxWidth: 80 }}
      />{' '}
      <TextField
        label="Minutes"
        size="small"
        type="number"
        defaultValue={minutes}
        inputRef={minutesField}
        onChange={triggerChangeDebounced}
        sx={{ maxWidth: 80 }}
      />{' '}
      <TextField
        label="Seconds"
        size="small"
        type="number"
        defaultValue={seconds}
        inputRef={secondsField}
        onChange={triggerChangeDebounced}
        sx={{ maxWidth: 80 }}
      />
    </div>
  );
};

export default EditDuration;
