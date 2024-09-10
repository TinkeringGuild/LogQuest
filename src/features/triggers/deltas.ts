import { pull, union } from 'lodash';
import { Trigger } from '../../generated/Trigger';
import { TriggerGroup } from '../../generated/TriggerGroup';
import { TriggerGroupDescendant } from '../../generated/TriggerGroupDescendant';
import { TriggerIndex } from '../../generated/TriggerIndex';
import { UUID } from '../../generated/UUID';
import { TriggerTag } from '../../generated/TriggerTag';

export function TriggerUpdated(index: TriggerIndex, trigger: Trigger) {
  index.triggers[trigger.id] = trigger;
}

export function TriggerGroupCreated(index: TriggerIndex, group: TriggerGroup) {
  index.groups[group.id] = group;
}

export function TriggerGroupChildrenChanged(
  index: TriggerIndex,
  value: {
    trigger_group_id: UUID;
    children: Array<TriggerGroupDescendant>;
  }
) {
  const group = index.groups[value.trigger_group_id];
  if (group) {
    group.children = value.children;
  }
}

export function TopLevelChanged(
  index: TriggerIndex,
  value: Array<TriggerGroupDescendant>
) {
  index.top_level = value;
}

export function TriggerTagged(
  index: TriggerIndex,
  value: { trigger_id: string; trigger_tag_id: string }
) {
  const { trigger_id, trigger_tag_id } = value;
  const trigger_tag = index.trigger_tags[trigger_tag_id];
  if (trigger_tag) {
    trigger_tag.triggers = union(trigger_tag.triggers, [trigger_id]);
  }
}

export function TriggerUntagged(
  index: TriggerIndex,
  value: { trigger_id: string; trigger_tag_id: string }
) {
  const { trigger_id, trigger_tag_id } = value;
  const trigger_tag = index.trigger_tags[trigger_tag_id];
  if (trigger_tag) {
    trigger_tag.triggers = pull(trigger_tag.triggers, trigger_id);
  }
}

export function TriggerTagCreated(
  index: TriggerIndex,
  trigger_tag: TriggerTag
) {
  index.trigger_tags[trigger_tag.id] = trigger_tag;
}

export function TriggerTagDeleted(index: TriggerIndex, trigger_tag_id: UUID) {
  delete index.trigger_tags[trigger_tag_id];
}