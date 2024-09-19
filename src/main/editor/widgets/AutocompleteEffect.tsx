import Autocomplete from '@mui/material/Autocomplete';
import TextField from '@mui/material/TextField';

import {
  EffectIcon,
  EFFECT_VARIANTS,
  EffectVariant,
  TimerEffectVariant,
  isTimerEffectVariant,
  HUMANIZED_EFFECT_NAMES,
  TimerEffectIcon,
  HUMANIZED_TIMER_EFFECT_NAMES,
  TIMER_EFFECT_VARIANTS,
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

export const VARIANTS_OF_EFFECTS_AND_TIMER_EFFECTS: (
  | EffectVariant
  | TimerEffectVariant
)[] = [...SHOWN_TIMER_EFFECT_VARIANTS, ...EFFECT_VARIANTS];

export const AutocompleteEffect: React.FC<{
  onSelect: (e: EffectVariant) => void;
  close?: () => void;
}> = ({ onSelect, close = () => {} }) => {
  return (
    <Autocomplete
      openOnFocus
      blurOnSelect
      autoHighlight
      size="small"
      options={EFFECT_VARIANTS}
      renderOption={(props, option, { selected: _ }) => {
        const { key, ...optionProps } = props;
        const VariantIcon = EffectIcon[option];
        return (
          <li key={key} {...optionProps}>
            <VariantIcon />
            &nbsp;&nbsp;{HUMANIZED_EFFECT_NAMES[option]}
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
};

export const AutocompleteEffectAndTimerEffect: React.FC<{
  onSelect: (e: EitherVariant) => void;
  close?: () => void;
}> = ({ onSelect, close = () => {} }) => {
  return (
    <Autocomplete
      openOnFocus
      blurOnSelect
      autoHighlight
      size="small"
      options={VARIANTS_OF_EFFECTS_AND_TIMER_EFFECTS}
      groupBy={(option: EitherVariant) =>
        isTimerEffectVariant(option) ? 'Timer Effects' : 'General Effects'
      }
      renderOption={(props, option, { selected: _ }) => {
        const { key, ...optionProps } = props;
        const VariantIcon = isTimerEffectVariant(option)
          ? TimerEffectIcon[option]
          : EffectIcon[option];
        return (
          <li key={key} {...optionProps}>
            <VariantIcon />
            &nbsp;&nbsp;
            {isTimerEffectVariant(option)
              ? HUMANIZED_TIMER_EFFECT_NAMES[option]
              : HUMANIZED_EFFECT_NAMES[option]}
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
      sx={{ width: 250 }}
    />
  );
};
