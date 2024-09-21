import { useEffect, useId } from 'react';
import { useDispatch } from 'react-redux';

import Alert from '@mui/material/Alert';

import {
  forgetError,
  setError,
} from '../../../features/triggers/triggerEditorSlice';

const EditorError: React.FC<{ message: string; center?: boolean }> = ({
  message: error,
  center,
}) => {
  const dispatch = useDispatch();
  const id = useId();

  useEffect(() => {
    dispatch(setError({ id, error }));
    return () => {
      dispatch(forgetError(id));
    };
  }, []);

  const sx = center ? { justifyContent: 'center' } : {};
  return (
    <Alert severity="error" sx={sx}>
      {error}
    </Alert>
  );
};

export default EditorError;
