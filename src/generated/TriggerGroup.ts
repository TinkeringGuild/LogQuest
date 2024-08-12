// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Timestamp } from "./Timestamp";
import type { TriggerGroupDescendant } from "./TriggerGroupDescendant";
import type { UUID } from "./UUID";

export type TriggerGroup = { id: UUID, name: string, comment: string | null, children: Array<TriggerGroupDescendant>, created_at: Timestamp, updated_at: Timestamp, };
