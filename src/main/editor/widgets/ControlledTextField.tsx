import { useEffect, useState } from 'react';

import TextField, { TextFieldProps } from '@mui/material/TextField';

const ControlledTextField: React.FC<
  TextFieldProps & {
    label: string;
    value: string;
    onCommit: (value: string) => void;
  }
> = ({ label, value, onCommit, ...props }) => {
  const [input, setInput] = useState(value);

  useEffect(() => {
    setInput(value);
  }, [value]);

  return (
    <TextField
      {...props}
      label={label}
      value={input}
      onChange={(e) => setInput(e.target.value)}
      onBlur={() => onCommit(input)}
    />
  );
};

export default ControlledTextField;
