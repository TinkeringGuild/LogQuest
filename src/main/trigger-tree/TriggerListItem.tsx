import { cloneDeep, map as pluck } from 'lodash';
import React, { useContext, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { v4 as uuid } from 'uuid';

import Switch from '@mui/material/Switch';

import {
  editNewTrigger,
  editTriggerDraft,
} from '../../features/triggers/triggerEditorSlice';
import {
  $groupsUptoGroup,
  $positionOf,
  $trigger,
  $triggerTagsHavingTrigger,
  applyDeltas,
} from '../../features/triggers/triggersSlice';
import { UUID } from '../../generated/UUID';
import {
  addTriggerToTag,
  createTrigger,
  createTriggerGroup,
  deleteTrigger,
  removeTriggerFromTag,
} from '../../ipc';
import store from '../../MainStore';
import { nowTimestamp } from '../../util';
import EmojiText from '../../widgets/EmojiText';
import TriggerGroupEditorDialog from './dialogs/TriggerGroupEditorDialog';
import TriggerContextMenu from './menus/TriggerContextMenu';
import TriggerIDsInSelectedTriggerTagContext from './TriggerIDsInSelectedTriggerTagContext';

const TriggerListItem: React.FC<{
  triggerID: UUID;
}> = ({ triggerID }) => {
  const dispatch = useDispatch();
  const trigger = useSelector($trigger(triggerID));

  const [contextMenuPosition, setContextMenuPosition] = useState<{
    top: number;
    left: number;
  } | null>(null);

  // If the value is null, then the dialog is not open
  const [createGroupDialogPositionParam, setCreateGroupDialogPositionParam] =
    useState<number | null>(null);

  const openContextMenu = (e: React.MouseEvent<HTMLSpanElement>) => {
    e.preventDefault();
    setContextMenuPosition({
      top: e.clientY + 2,
      left: e.clientX + 2,
    });
  };

  return (
    <li
      className={`view-trigger-list-item ${contextMenuPosition ? 'view-trigger-list-item-context-menu-open' : ''}`}
    >
      <TriggerTagInclusionSwitch triggerID={triggerID} />{' '}
      <a
        href="#"
        className="view-trigger-list-item-name"
        onContextMenu={openContextMenu}
        onClick={(e) => {
          e.preventDefault();
          dispatch(
            editTriggerDraft({
              trigger: cloneDeep(trigger),
              triggerTags: $triggerTagsHavingTrigger(trigger.id)(
                store.getState()
              ),
            })
          );
        }}
      >
        <EmojiText text={trigger.name} />
      </a>
      {contextMenuPosition && (
        <TriggerContextMenu
          onDuplicate={async () => {
            const state = store.getState();
            const tags = $triggerTagsHavingTrigger(trigger.id)(state);
            const thisPosition = $positionOf({ trigger: trigger.id })(state);
            const now = nowTimestamp();
            const duplicate = {
              ...trigger,
              id: uuid(),
              name: trigger.name + ' (DUPLICATE)',
              created_at: now,
              updated_at: now,
            };
            const deltas = await createTrigger(
              duplicate,
              pluck(tags, 'id'),
              thisPosition + 1
            );
            dispatch(applyDeltas(deltas));
          }}
          onInsertTrigger={(offset) => {
            const state = store.getState();
            const ancestorGroups = $groupsUptoGroup(trigger.parent_id)(state);
            const positionOfTrigger = $positionOf({
              trigger: trigger.id,
            })(state);
            const parentPosition = positionOfTrigger + offset;
            dispatch(
              editNewTrigger({
                parentID: trigger.parent_id,
                parentPosition,
                ancestorGroups,
              })
            );
          }}
          onInsertGroup={(offset) => {
            const thisPosition = $positionOf({ trigger: trigger.id })(
              store.getState()
            );
            setCreateGroupDialogPositionParam(thisPosition + offset);
          }}
          onDelete={async () => {
            const deltas = await deleteTrigger(triggerID);
            dispatch(applyDeltas(deltas));
          }}
          close={() => setContextMenuPosition(null)}
          {...contextMenuPosition}
        />
      )}
      {createGroupDialogPositionParam !== null && (
        <TriggerGroupEditorDialog
          name=""
          comment={null}
          onSave={async (name, comment) => {
            const deltas = createTriggerGroup(
              name,
              comment,
              trigger.parent_id,
              createGroupDialogPositionParam
            );
            dispatch(applyDeltas(await deltas));
          }}
          close={() => setCreateGroupDialogPositionParam(null)}
        />
      )}
    </li>
  );
};

const TriggerTagInclusionSwitch: React.FC<{ triggerID: UUID }> = ({
  triggerID,
}) => {
  const dispatch = useDispatch();

  const activeTriggers = useContext(TriggerIDsInSelectedTriggerTagContext);

  if (!activeTriggers) {
    return <></>;
  }

  const checked = activeTriggers.triggerIDs.has(triggerID);

  return (
    <Switch
      size="small"
      checked={checked}
      className="toggle-trigger-tag-inclusion-switch"
      onChange={({ target: { checked } }) => {
        if (checked) {
          addTriggerToTag(triggerID, activeTriggers.tagID).then((deltas) =>
            dispatch(applyDeltas(deltas))
          );
        } else {
          removeTriggerFromTag(triggerID, activeTriggers.tagID).then((deltas) =>
            dispatch(applyDeltas(deltas))
          );
        }
      }}
    />
  );
};

export default React.memo(TriggerListItem);
