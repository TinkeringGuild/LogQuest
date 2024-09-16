import Autocomplete from '@mui/material/Autocomplete';

import { Effect } from '../../../generated/Effect';
import TextField from '@mui/material/TextField';

type EffectVariant = Effect['variant'];

const EFFECTS: EffectVariant[] = [
  'StartTimer',
  // "StartStopwatch"
  'OverlayMessage',
  'CopyToClipboard',
  'PlayAudioFile',
  'Speak',
  'SpeakStop',
  'RunSystemCommand',
  'Pause',
  'Sequence',
  'Parallel',
  'DoNothing',
  // 'ScopedTimerEffect',
];

const humanizeEffectVariant: (name: EffectVariant) => string = (name) => {
  switch (name) {
    case 'Sequence':
    case 'Parallel':
    case 'Pause':
    case 'Speak':
      return name;
    case 'StartTimer':
      return 'Start Timer';
    case 'CopyToClipboard':
      return 'Copy to Clipboad';
    case 'DoNothing':
      return 'Do Nothing';
    case 'OverlayMessage':
      return 'Overlay Message';
    case 'PlayAudioFile':
      return 'Play Audio File';
    case 'RunSystemCommand':
      return 'Run System Command';
    case 'ScopedTimerEffect':
      return 'Timer Effect';
    case 'SpeakStop':
      return 'Stop Speaking';
    case 'StartStopwatch':
      return 'Start Stopwatch';
  }
};

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
    getOptionLabel={humanizeEffectVariant}
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
