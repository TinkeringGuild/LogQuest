// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CommandTemplateSecurityCheck } from './CommandTemplateSecurityCheck';
import type { Duration } from './Duration';
import type { Stopwatch } from './Stopwatch';
import type { TemplateString } from './TemplateString';
import type { Timer } from './Timer';
import type { TimerEffect } from './TimerEffect';

export type Effect =
  | { variant: 'Parallel'; value: Array<Effect> }
  | { variant: 'Sequence'; value: Array<Effect> }
  | { variant: 'PlayAudioFile'; value: TemplateString | null }
  | { variant: 'CopyToClipboard'; value: TemplateString }
  | { variant: 'OverlayMessage'; value: TemplateString }
  | { variant: 'StartTimer'; value: Timer }
  | { variant: 'StartStopwatch'; value: Stopwatch }
  | { variant: 'RunSystemCommand'; value: CommandTemplateSecurityCheck }
  | { variant: 'SpeakStop' }
  | { variant: 'Speak'; value: { tmpl: TemplateString; interrupt: boolean } }
  | { variant: 'ScopedTimerEffect'; value: TimerEffect }
  | { variant: 'Pause'; value: Duration }
  | { variant: 'DoNothing' };
