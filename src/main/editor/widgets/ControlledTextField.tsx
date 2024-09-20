import { omit } from 'lodash';
import { useEffect, useId, useMemo, useState } from 'react';
import { useDispatch } from 'react-redux';

import TextField, { TextFieldProps } from '@mui/material/TextField';
import {
  forgetError,
  setError,
} from '../../../features/triggers/triggerEditorSlice';

type ControlledTextFieldValidationProps = {
  validate: (value: string) => string | null;
  // onValidateChange: (errorMessage: string | null) => void;
  errorID?: string;
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
  const dispatch = useDispatch();
  const [input, setInput] = useState(value);
  const defaultID = useId();

  const errorID = 'errorID' in props ? props.errorID! : defaultID;

  const validate = 'validate' in props && props.validate;
  // const onValidateChange = 'onValidateChange' in props && props.onValidateChange;

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

  // Register errors when validation state changes
  useEffect(() => {
    if (validation === undefined) {
      return;
    }
    if (validation) {
      dispatch(setError({ id: errorID, error: validation }));
    } else {
      dispatch(forgetError(errorID));
    }

    // onValidateChange && onValidateChange(validation);
  }, [validation]);

  // Clean up error on un-mount
  useEffect(
    () => () => {
      dispatch(forgetError(errorID));
    },
    []
  );

  return (
    <TextField
      {...omit(props, ['validate', 'errorID' /*, 'onValidateChange' */])}
      id={defaultID}
      label={label}
      value={input}
      error={!!validation}
      helperText={validation || undefined}
      onChange={(e) => setInput(e.target.value)}
      onBlur={() => onCommit(input)}
    />
  );
};

export default ControlledTextField;
