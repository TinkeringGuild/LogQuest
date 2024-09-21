import AlarmOnOutlined from '@mui/icons-material/AlarmOnOutlined';
import AvTimer from '@mui/icons-material/AvTimer';
import ContentPasteOutlined from '@mui/icons-material/ContentPasteOutlined';
import FormatAlignLeft from '@mui/icons-material/FormatAlignLeft';
import HideSourceOutlined from '@mui/icons-material/HideSourceOutlined';
import HourglassBottomOutlined from '@mui/icons-material/HourglassBottomOutlined';
import HourglassTopOutlined from '@mui/icons-material/HourglassTopOutlined';
import InsertCommentOutlined from '@mui/icons-material/InsertCommentOutlined';
import KeyboardDoubleArrowDownOutlined from '@mui/icons-material/KeyboardDoubleArrowDownOutlined';
import LabelOffSharp from '@mui/icons-material/LabelOffSharp';
import LabelSharp from '@mui/icons-material/LabelSharp';
import PauseCircleOutline from '@mui/icons-material/PauseCircleOutline';
import QuestionMark from '@mui/icons-material/QuestionMark';
import RecordVoiceOverOutlined from '@mui/icons-material/RecordVoiceOverOutlined';
import RestartAltOutlined from '@mui/icons-material/RestartAltOutlined';
import TerminalSharp from '@mui/icons-material/TerminalSharp';
import TimerOffOutlined from '@mui/icons-material/TimerOffOutlined';
import VisibilityOffOutlined from '@mui/icons-material/VisibilityOffOutlined';
import VisibilityOutlined from '@mui/icons-material/VisibilityOutlined';
import VoiceOverOffOutlined from '@mui/icons-material/VoiceOverOffOutlined';
import VolumeUpOutlined from '@mui/icons-material/VolumeUpOutlined';
import WatchLater from '@mui/icons-material/WatchLater';

import { Effect } from '../../generated/Effect';
import { TimerEffect } from '../../generated/TimerEffect';
import { ReactElement } from 'react';
import React from 'react';

export type EffectVariant = Effect['variant'];

export type TimerEffectVariant = TimerEffect['variant'];

export const EFFECT_VARIANTS: EffectVariant[] = [
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

export const TIMER_EFFECT_VARIANTS: TimerEffectVariant[] = [
  'ClearTimer',
  'RestartTimer',
  'HideTimer',
  'UnhideTimer',
  'WaitUntilFilterMatches',
  'WaitUntilSecondsRemain',
  'WaitUntilFinished',

  'AddTag',
  'RemoveTag',
  'IncrementCounter',
  'DecrementCounter',
  'ResetCounter',
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
  ScopedTimerEffect: QuestionMark,
};

export const TimerEffectIcon: {
  [key in TimerEffectVariant]: React.ComponentType;
} = {
  AddTag: LabelSharp,
  ClearTimer: TimerOffOutlined,
  HideTimer: VisibilityOffOutlined,
  RemoveTag: LabelOffSharp,
  RestartTimer: RestartAltOutlined,
  UnhideTimer: VisibilityOutlined,
  WaitUntilFilterMatches: HourglassTopOutlined,
  WaitUntilFinished: AlarmOnOutlined,
  WaitUntilSecondsRemain: HourglassBottomOutlined,

  // THESE ARE NOT YET IMPLEMENTED
  IncrementCounter: QuestionMark,
  DecrementCounter: QuestionMark,
  ResetCounter: QuestionMark,
};

const HUMANIZED_TIMER_EFFECT_NAMES: {
  [key in TimerEffectVariant]: string;
} = {
  AddTag: 'Add Timer Tag',
  ClearTimer: 'Clear this Timer',
  HideTimer: 'Hide this Timer',
  RemoveTag: 'Remove Timer Tag',
  RestartTimer: 'Restart this Timer',
  UnhideTimer: 'Un-Hide this Timer',
  WaitUntilFilterMatches: 'Wait until Filter Matches',
  WaitUntilFinished: 'Wait until Finished',
  WaitUntilSecondsRemain: 'Wait until Seconds Remain',

  // THESE ARE NOT YET IMPLEMENTED
  IncrementCounter: 'Increment Counter',
  DecrementCounter: 'Decrement Counter',
  ResetCounter: 'Reset Counter',
};

const HUMANIZED_EFFECT_NAMES: { [key in EffectVariant]: string } = {
  Sequence: 'Sequence',
  Parallel: 'Parallel',
  Pause: 'Pause',
  Speak: 'Speak',
  StartTimer: 'Start Timer',
  CopyToClipboard: 'Copy to Clipboard',
  DoNothing: 'Do Nothing',
  OverlayMessage: 'Overlay Message',
  PlayAudioFile: 'Play Audio File',
  RunSystemCommand: 'System Command',
  ScopedTimerEffect: 'Timer Effect',
  SpeakStop: 'Stop Speaking',
  StartStopwatch: 'Start Stopwatch',
};

export function humanizeEffectName(
  variant: EffectVariant | TimerEffectVariant
): string {
  return isTimerEffectVariant(variant)
    ? HUMANIZED_TIMER_EFFECT_NAMES[variant]
    : HUMANIZED_EFFECT_NAMES[variant];
}

export function effectIcon(
  variant: EffectVariant | TimerEffectVariant
): ReactElement {
  const icon = isTimerEffectVariant(variant)
    ? TimerEffectIcon[variant]
    : EffectIcon[variant];
  return React.createElement(icon);
}

export function isTimerEffectVariant(
  variant: string
): variant is TimerEffectVariant {
  return TIMER_EFFECT_VARIANTS.includes(variant as TimerEffectVariant);
}
