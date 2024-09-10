import { DeleteForeverOutlined, PlaylistAdd } from '@mui/icons-material';
import Button from '@mui/material/Button';
import FormControl from '@mui/material/FormControl';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';
import InputLabel from '@mui/material/InputLabel';
import OutlinedInput from '@mui/material/OutlinedInput';
import Stack from '@mui/material/Stack';
import { useDispatch, useSelector } from 'react-redux';

import {
  appendNewMatcher,
  deleteFilterMatcher,
  editorSelector,
  EditorSelector,
  setMatcherValue,
} from '../../../features/triggers/editorSlice';
import { Filter } from '../../../generated/Filter';
import { FilterWithContext } from '../../../generated/FilterWithContext';
import { Matcher } from '../../../generated/Matcher';
import { MatcherWithContext } from '../../../generated/MatcherWithContext';
import StandardTooltip from '../../../widgets/StandardTooltip';

import './EditFilter.css';

type MatcherVariant = (Matcher & MatcherWithContext)['variant'];

const MatcherInputField: React.FC<{
  variant: MatcherVariant;
  defaultValue: string;
  onDelete: () => void;
  onBlur: (value: string) => void;
}> = ({ defaultValue, variant, onDelete, onBlur }) => {
  const variantHumanized = humanizeMatcherVariant(variant);
  return (
    <FormControl sx={{ m: 1 }} variant="outlined">
      <InputLabel>{variantHumanized}</InputLabel>
      <OutlinedInput
        label={variantHumanized}
        defaultValue={defaultValue}
        className="pattern-input"
        type="text"
        fullWidth
        multiline
        onBlur={(e) => onBlur(e.target.value)}
        endAdornment={
          <InputAdornment position="end">
            <StandardTooltip help="Delete this pattern">
              <IconButton edge="end" onClick={onDelete}>
                <DeleteForeverOutlined />
              </IconButton>
            </StandardTooltip>
          </InputAdornment>
        }
      />
    </FormControl>
  );
};

function EditFilter<T extends Filter | FilterWithContext>({
  selector,
}: {
  selector: EditorSelector<T>;
}): JSX.Element {
  const dispatch = useDispatch();
  const filter = useSelector(editorSelector(selector));

  return (
    <Stack spacing={2}>
      {filter.map((matcher, index) => (
        <MatcherInputField
          key={index}
          defaultValue={matcher.value}
          variant="GINA"
          onDelete={() => dispatch(deleteFilterMatcher({ index, selector }))}
          onBlur={(value) =>
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
          dispatch(appendNewMatcher({ selector, matcherVariant: 'GINA' }));
        }}
      >
        Add a new Pattern
      </Button>
    </Stack>
  );
}

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
