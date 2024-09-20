import { ReactNode, useContext, useState } from 'react';

import Add from '@mui/icons-material/Add';
import ArrowDownward from '@mui/icons-material/ArrowDownward';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';

import {
  createEffectAutocomplete,
  createEffectOrTimerEffectAutocomplete,
} from './AutocompleteEffect';
import IncludeTimerEffectsContext from './IncludeTimerEffectsContext';

const InsertEffectDivider: React.FC<{
  index: number;
  defaultIcon: ReactNode;
  onInsertEffect: (effect: string, index: number) => void;
}> = ({ index, defaultIcon, onInsertEffect }) => {
  const [hovered, setHovered] = useState(false);
  const [filterModeActive, setFilterModeActive] = useState(false);

  const includeTimerEffects = useContext(IncludeTimerEffectsContext);

  return (
    <Divider
      className="insert-effect-divider"
      sx={{ height: 41 }}
      onMouseOver={() => setHovered(true)}
      onMouseOut={() => setHovered(false)}
    >
      {filterModeActive ? (
        includeTimerEffects ? (
          createEffectOrTimerEffectAutocomplete({
            onSelect: (variant) => onInsertEffect(variant, index),
            close: () => setFilterModeActive(false),
          })
        ) : (
          createEffectAutocomplete({
            onSelect: (variant) => onInsertEffect(variant, index),
            close: () => setFilterModeActive(false),
          })
        )
      ) : hovered ? (
        <Button
          onClick={() => setFilterModeActive(true)}
          startIcon={hovered ? <Add /> : <ArrowDownward />}
          sx={{ color: 'black' }}
        >
          Insert New Effect Here
        </Button>
      ) : (
        <IconButton>{defaultIcon}</IconButton>
      )}
    </Divider>
  );
};

export default InsertEffectDivider;
