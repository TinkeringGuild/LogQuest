import {
  ReactNode,
  RefCallback,
  useEffect,
  useId,
  useRef,
  useState,
} from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { DeleteForeverOutlined, PlaylistAdd } from '@mui/icons-material';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';

import {
  $errorForID,
  appendNewMatcher,
  deleteFilterMatcher,
  forgetError,
  setError,
  setMatcherValue,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../../features/triggers/triggerEditorSlice';
import { Filter } from '../../../generated/Filter';
import { FilterWithContext } from '../../../generated/FilterWithContext';
import { Matcher } from '../../../generated/Matcher';
import { MatcherWithContext } from '../../../generated/MatcherWithContext';
import { validateGINARegex, ValidateGINARegexResponse } from '../../../ipc';
import StandardTooltip from '../../../widgets/StandardTooltip';

import './EditFilter.css';

type MatcherVariant = (Matcher & MatcherWithContext)['variant'];

function EditFilter<T extends Filter | FilterWithContext>({
  selector,
  children,
}: {
  selector: TriggerEditorSelector<T>;
  children?: ReactNode;
}): JSX.Element {
  const dispatch = useDispatch();
  const filter = useSelector(triggerEditorSelector(selector));

  const matcherInputFieldRefs = useRef<HTMLInputElement[]>([]);
  const [isAdding, setIsAdding] = useState(false);

  useEffect(() => {
    if (isAdding && matcherInputFieldRefs.current.length > 0) {
      const lastIndex = matcherInputFieldRefs.current.length - 1;
      matcherInputFieldRefs.current[lastIndex].focus();
      setIsAdding(false);
    }
  }, [isAdding]);

  return (
    <Stack spacing={2}>
      {!filter.length
        ? children
        : filter.map((matcher, index) => (
            <MatcherInputField
              key={matcher.value.id}
              value={matcher.value.pattern}
              variant={matcher.variant}
              getRef={(ref) =>
                ref && (matcherInputFieldRefs.current[index] = ref)
              }
              onDelete={() => {
                matcherInputFieldRefs.current.splice(index, 1);
                dispatch(deleteFilterMatcher({ index, selector }));
              }}
              onChange={(value) =>
                dispatch(
                  setMatcherValue({
                    value,
                    selector: (slice) => selector(slice)[index],
                  })
                )
              }
            />
          ))}
      <Button
        variant="outlined"
        size="large"
        startIcon={<PlaylistAdd />}
        onClick={() => {
          setIsAdding(true);
          dispatch(appendNewMatcher({ selector, variant: 'GINA' }));
        }}
      >
        Add a new Pattern
      </Button>
    </Stack>
  );
}

const MatcherInputField: React.FC<{
  variant: MatcherVariant;
  value: string;
  getRef: RefCallback<HTMLInputElement>;
  onDelete: () => void;
  onChange: (value: string) => void;
}> = ({ value, variant, getRef, onDelete, onChange }) => {
  const dispatch = useDispatch();

  const id = useId();
  const [pattern, setPattern] = useState('');

  // Keeps pattern in-sync with the prop passed in
  useEffect(() => {
    setPattern(value);
  }, [value]);

  // Cleanup error state on un-mount
  useEffect(() => {
    return () => {
      dispatch(forgetError(id));
    };
  }, []);

  // Validates GINA regex patterns via a Tauri command
  useEffect(() => {
    if (variant !== 'GINA') {
      return;
    }
    let isMounted = true;

    if (pattern.trim()) {
      validateGINARegex(pattern).then(
        (errorMaybe: ValidateGINARegexResponse) => {
          if (!isMounted) {
            return;
          }
          if (errorMaybe) {
            // I cannot use the position in the error message (yet) because a RegexGINA
            // interpolates data into the Regex, making the position number somewhat useless.
            // I would have to fix this in Rust, intercepting parse errors and subtracting
            // out the length of the interpolated sections... that's low-priority for now.
            const [_positionMaybe, error] = errorMaybe;
            setRegexError(error);
          } else {
            setRegexError(null);
          }
        }
      );
    } else {
      setRegexError('Pattern cannot be blank');
    }

    return () => {
      isMounted = false;
    };
  }, [pattern]);

  const regexError: string | undefined = useSelector($errorForID(id));

  const setRegexError = (error: string | null) => {
    const action = error ? setError({ id, error }) : forgetError(id);
    dispatch(action);
  };

  const variantHumanized = humanizeMatcherVariant(variant);

  return (
    <TextField
      className="pattern-input template-input"
      label={variantHumanized}
      value={pattern}
      id={id}
      variant="outlined"
      error={!!regexError}
      helperText={regexError}
      fullWidth
      multiline
      inputRef={getRef}
      onChange={(e) => setPattern(e.target.value)}
      onBlur={() => onChange(pattern)}
      slotProps={{
        input: {
          endAdornment: (
            <InputAdornment position="end">
              <StandardTooltip help="Delete this pattern">
                <IconButton edge="end" onClick={onDelete}>
                  <DeleteForeverOutlined />
                </IconButton>
              </StandardTooltip>
            </InputAdornment>
          ),
        },
      }}
    />
  );
};

const humanizeMatcherVariant = (variant: MatcherVariant) => {
  if (variant === 'WholeLine') {
    return 'Whole Line';
  } else if (variant === 'PartialLine') {
    return 'Partial Line';
  } else if (variant === 'GINA') {
    return 'GINA-style Regular Expression';
  } else if (variant === 'Pattern') {
    return 'LogQuest-style Regular Expression';
  } else {
    // Shouldn't ever get here
    return variant;
  }
};

export default EditFilter;
