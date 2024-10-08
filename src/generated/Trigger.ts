// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { EffectWithID } from './EffectWithID';
import type { Filter } from './Filter';
import type { Timestamp } from './Timestamp';
import type { UUID } from './UUID';

export type Trigger = {
  id: UUID;
  parent_id: UUID | null;
  name: string;
  comment: string | null;
  filter: Filter;
  effects: Array<EffectWithID>;
  created_at: Timestamp;
  updated_at: Timestamp;
};
