import { useEffect, useMemo, useState } from 'react';

import TextField, { TextFieldProps } from '@mui/material/TextField';
import { omit } from 'lodash';

type ControlledTextFieldValidationProps = {
  validate: (value: string) => string | null;
  onValidateChange: (errorMessage: string | null) => void;
};

type BaseControlledTextFieldProps = TextFieldProps & {
  label: string;
  value: string;
  onCommit: (value: string) => void;
};

type ControlledTextFieldPropsWithValidationProps =
  BaseControlledTextFieldProps & ControlledTextFieldValidationProps;

// TODO: For some reason, this type definition doesn't strictly enforce
// both `validate` and `onValidateChange` being required together
type ControlledTextFieldProps =
  | BaseControlledTextFieldProps
  | ControlledTextFieldPropsWithValidationProps;

const ControlledTextField: React.FC<ControlledTextFieldProps> = ({
  label,
  value,
  onCommit,
  ...props
}) => {
  const [input, setInput] = useState(value);

  const [validate, onValidateChange] =
    'validate' in props && 'onValidateChange' in props
      ? [props.validate, props.onValidateChange]
      : [null, null];

  // Synchronize the component state with the value prop
  useEffect(() => {
    setInput(value);
  }, [value]);

  const validation: string | null | undefined = useMemo(() => {
    if (validate) {
      return validate(input);
    } else {
      return undefined;
    }
  }, [input]);

  // Invoke onValidateChange when the validation state changes
  useEffect(() => {
    if (validation !== undefined && onValidateChange) {
      onValidateChange(validation);
    }
  }, [validation]);

  return (
    <TextField
      {...omit(props, ['validate', 'onValidateChange'])}
      label={label}
      value={input}
      onChange={(e) => setInput(e.target.value)}
      onBlur={() => onCommit(input)}
    />
  );
};

export default ControlledTextField;
