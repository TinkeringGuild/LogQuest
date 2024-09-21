import { useEffect, useId } from 'react';
import { useDispatch } from 'react-redux';

import Alert from '@mui/material/Alert';

import {
  forgetError,
  setError,
} from '../../../features/triggers/triggerEditorSlice';

const EditorError: React.FC<{
  message: string;
  center?: boolean;
  width?: number;
}> = ({ message: error, center, width }) => {
  const dispatch = useDispatch();
  const id = useId();

  useEffect(() => {
    dispatch(setError({ id, error }));
    return () => {
      dispatch(forgetError(id));
    };
  }, [error]);

  const sxCenter = center ? { justifyContent: 'center' } : {};
  const sxWidth = width ? { width } : {};

  return (
    <Alert severity="error" sx={{ ...sxCenter, ...sxWidth }}>
      {error}
    </Alert>
  );
};

export default EditorError;
