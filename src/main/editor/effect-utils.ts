import AvTimer from '@mui/icons-material/AvTimer';
import ContentPasteOutlined from '@mui/icons-material/ContentPasteOutlined';
import FormatAlignLeft from '@mui/icons-material/FormatAlignLeft';
import HideSourceOutlined from '@mui/icons-material/HideSourceOutlined';
import InsertCommentOutlined from '@mui/icons-material/InsertCommentOutlined';
import KeyboardDoubleArrowDownOutlined from '@mui/icons-material/KeyboardDoubleArrowDownOutlined';
import PauseCircleOutline from '@mui/icons-material/PauseCircleOutline';
import RecordVoiceOverOutlined from '@mui/icons-material/RecordVoiceOverOutlined';
import TerminalSharp from '@mui/icons-material/TerminalSharp';
import VoiceOverOffOutlined from '@mui/icons-material/VoiceOverOffOutlined';
import VolumeUpOutlined from '@mui/icons-material/VolumeUpOutlined';
import WatchLater from '@mui/icons-material/WatchLater';

import { Effect } from '../../generated/Effect';

export type EffectVariant = Effect['variant'];

export const EFFECTS: EffectVariant[] = [
  'StartTimer',
  // "StartStopwatch"
  'OverlayMessage',
  'CopyToClipboard',
  'PlayAudioFile',
  'Speak',
  'SpeakStop',
  'RunSystemCommand',
  'Sequence',
  'Parallel',
  'Pause',
  'DoNothing',
  // 'ScopedTimerEffect',
];

export const EffectIcon: { [key in EffectVariant]: React.ComponentType } = {
  CopyToClipboard: ContentPasteOutlined,
  DoNothing: HideSourceOutlined,
  OverlayMessage: InsertCommentOutlined,
  Parallel: FormatAlignLeft,
  Pause: PauseCircleOutline,
  PlayAudioFile: VolumeUpOutlined,
  RunSystemCommand: TerminalSharp,
  Sequence: KeyboardDoubleArrowDownOutlined,
  Speak: RecordVoiceOverOutlined,
  SpeakStop: VoiceOverOffOutlined,
  StartStopwatch: WatchLater,
  StartTimer: AvTimer,

  // ScopedTimerEffect isn't shown like normal Effects, so this icon isn't used
  // but it's included here because of the TypeScript completeness check on EffectIcon
  ScopedTimerEffect: AvTimer,
};

export function humanizeEffectVariant(name: EffectVariant): string {
  switch (name) {
    case 'Sequence':
    case 'Parallel':
    case 'Pause':
    case 'Speak':
      return name;
    case 'StartTimer':
      return 'Start Timer';
    case 'CopyToClipboard':
      return 'Copy to Clipboard';
    case 'DoNothing':
      return 'Do Nothing';
    case 'OverlayMessage':
      return 'Overlay Message';
    case 'PlayAudioFile':
      return 'Play Audio File';
    case 'RunSystemCommand':
      return 'System Command';
    case 'ScopedTimerEffect':
      return 'Timer Effect';
    case 'SpeakStop':
      return 'Stop Speaking';
    case 'StartStopwatch':
      return 'Start Stopwatch';
  }
}
