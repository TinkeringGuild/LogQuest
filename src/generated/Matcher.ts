// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { RegexGINA } from './RegexGINA';
import type { SerializableRegex } from './SerializableRegex';
import type { UUID } from './UUID';

export type Matcher =
  | { variant: 'WholeLine'; value: { id: UUID; pattern: string } }
  | { variant: 'PartialLine'; value: { id: UUID; pattern: string } }
  | { variant: 'Pattern'; value: { id: UUID; pattern: SerializableRegex } }
  | { variant: 'GINA'; value: { id: UUID; pattern: RegexGINA } };
