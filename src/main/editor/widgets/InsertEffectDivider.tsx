import { ReactNode, useState } from 'react';

import Add from '@mui/icons-material/Add';
import ArrowDownward from '@mui/icons-material/ArrowDownward';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';

import { EffectVariant } from '../effect-utils';
import { AutocompleteEffect } from './AutocompleteEffect';

const InsertEffectDivider: React.FC<{
  index: number;
  defaultIcon: ReactNode;
  onInsertEffect: (effect: EffectVariant, index: number) => void;
}> = ({ index, onInsertEffect, defaultIcon }) => {
  const [hovered, setHovered] = useState(false);
  const [filterModeActive, setFilterModeActive] = useState(false);

  return (
    <Divider
      className="insert-effect-divider"
      sx={{ height: 41 }}
      onMouseOver={() => setHovered(true)}
      onMouseOut={() => setHovered(false)}
    >
      {filterModeActive ? (
        <AutocompleteEffect
          onSelect={(variant) => {
            setHovered(false);
            onInsertEffect(variant, index);
          }}
          close={() => setFilterModeActive(false)}
        />
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
