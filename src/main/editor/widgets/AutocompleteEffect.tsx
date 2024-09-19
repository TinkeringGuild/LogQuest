import Autocomplete from '@mui/material/Autocomplete';
import TextField from '@mui/material/TextField';

import {
  EffectIcon,
  EFFECTS,
  EffectVariant,
  humanizeEffectVariant,
} from '../effect-utils';

const AutocompleteEffect: React.FC<{
  includeTimerEffects?: boolean;
  onSelect: (e: EffectVariant) => void;
  close?: () => void;
}> = ({ onSelect, close = () => {} }) => (
  <Autocomplete
    openOnFocus
    blurOnSelect
    autoHighlight
    size="small"
    options={EFFECTS}
    renderOption={(props, option, { selected: _ }) => {
      const { key, ...optionProps } = props;
      const VariantIcon = EffectIcon[option];
      return (
        <li key={key} {...optionProps}>
          <VariantIcon />
          &nbsp;&nbsp;{humanizeEffectVariant(option)}
        </li>
      );
    }}
    onChange={(_, value) => {
      if (value) {
        onSelect(value);
      }
      close();
    }}
    renderInput={(params) => (
      <TextField {...params} autoFocus={true} onBlur={close} />
    )}
    sx={{ width: 200 }}
  />
);

export default AutocompleteEffect;
