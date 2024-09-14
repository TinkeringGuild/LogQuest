import { cloneDeep } from 'lodash';
import React, { useContext, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import Switch from '@mui/material/Switch';

import {
  editNewTrigger,
  editTriggerDraft,
} from '../../features/triggers/triggerEditorSlice';
import {
  $groupsUptoGroup,
  $positionOfTrigger,
  $trigger,
  $triggerTagsHavingTrigger,
  applyDeltas,
} from '../../features/triggers/triggersSlice';
import { UUID } from '../../generated/UUID';
import {
  addTriggerToTag,
  deleteTrigger,
  removeTriggerFromTag,
} from '../../ipc';
import store from '../../MainStore';
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
        {trigger.name}
      </a>
      {contextMenuPosition && (
        <TriggerContextMenu
          onInsertTrigger={(offset) => {
            const storeState = store.getState();
            const ancestorGroups = $groupsUptoGroup(trigger.parent_id)(
              storeState
            );
            const positionOfTrigger = $positionOfTrigger(trigger.id)(
              storeState
            );
            const parentPosition = positionOfTrigger + offset;
            dispatch(
              editNewTrigger({
                parentID: trigger.parent_id,
                parentPosition,
                ancestorGroups,
              })
            );
          }}
          onDelete={async () => {
            const deltas = await deleteTrigger(triggerID);
            dispatch(applyDeltas(deltas));
          }}
          close={() => setContextMenuPosition(null)}
          {...contextMenuPosition}
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
