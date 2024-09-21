import { ReactElement } from 'react';

import Autocomplete from '@mui/material/Autocomplete';
import TextField from '@mui/material/TextField';

import {
  EFFECT_VARIANTS,
  EffectVariant,
  TIMER_EFFECT_VARIANTS,
  TimerEffectVariant,
  effectIcon,
  humanizeEffectName,
  isTimerEffectVariant,
} from '../effect-utils';

type EitherVariant = EffectVariant | TimerEffectVariant;

const SHOWN_TIMER_EFFECT_VARIANTS: TimerEffectVariant[] =
  TIMER_EFFECT_VARIANTS.filter(
    (variant) =>
      ![
        'IncrementCounter',
        'DecrementCounter',
        'ResetCounter',
        'AddTag',
        'RemoveTag',
      ].includes(variant)
  );

export const VARIANTS_OF_TIMER_EFFECTS_AND_EFFECTS: (
  | EffectVariant
  | TimerEffectVariant
)[] = [...SHOWN_TIMER_EFFECT_VARIANTS, ...EFFECT_VARIANTS];

export const createAutocomplete = <
  OptionType extends EffectVariant | EitherVariant,
>(props: {
  options: OptionType[];
  groupBy?: (value: OptionType) => string;
  onSelect: (value: OptionType) => void;
  close?: () => void;
  width: number;
}): ReactElement => {
  const { onSelect, close, width, ...autocompleteProps } = props;
  return (
    <Autocomplete<OptionType>
      {...autocompleteProps}
      openOnFocus
      blurOnSelect
      autoHighlight
      size="small"
      renderOption={(props, option: OptionType) => {
        const { key, ...optionProps } = props;
        const name = humanizeEffectName(option);
        const icon = effectIcon(option);

        return (
          <li key={key} {...optionProps}>
            {icon}
            &nbsp;&nbsp;{name}
          </li>
        );
      }}
      onChange={(_, value) => {
        value && onSelect(value);
        close && close();
      }}
      renderInput={(params) => (
        <TextField {...params} autoFocus={true} onBlur={close} />
      )}
      sx={{ width: width }}
    />
  );
};

export const createEffectAutocomplete = (props: {
  onSelect: (value: EffectVariant) => void;
  close?: () => void;
}) => createAutocomplete({ width: 200, options: EFFECT_VARIANTS, ...props });

export const createEffectOrTimerEffectAutocomplete = (props: {
  onSelect: (value: EitherVariant) => void;
  close?: () => void;
}) =>
  createAutocomplete({
    options: VARIANTS_OF_TIMER_EFFECTS_AND_EFFECTS,
    width: 275,
    groupBy: (option) =>
      isTimerEffectVariant(option) ? 'Timer Effects' : 'General Effects',
    ...props,
  });
