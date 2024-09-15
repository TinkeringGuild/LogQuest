import React, { useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { editNewTrigger } from '../../features/triggers/triggerEditorSlice';
import {
  $groupsUptoGroup,
  $positionOf,
  $triggerGroup,
  applyDeltas,
} from '../../features/triggers/triggersSlice';
import { UUID } from '../../generated/UUID';
import {
  createTriggerGroup,
  deleteTriggerGroup,
  saveTriggerGroup,
} from '../../ipc';
import store from '../../MainStore';
import TriggerGroupEditorDialog from './dialogs/TriggerGroupEditorDialog';
import TriggerGroupContextMenu from './menus/TriggerGroupContextMenu';
import TriggerListItem from './TriggerListItem';

type GroupEditorCreateState = { position: number };

const TriggerGroupListItem: React.FC<{
  groupID: UUID;
}> = ({ groupID }) => {
  const dispatch = useDispatch();
  const group = useSelector($triggerGroup(groupID));

  const [editDialogState, setEditDialogState] = useState<
    'closed' | 'edit' | GroupEditorCreateState
  >('closed');

  const [contextMenuPosition, setContextMenuPosition] = useState<{
    top: number;
    left: number;
  } | null>(null);

  const openContextMenu = (event: React.MouseEvent<HTMLSpanElement>) => {
    event.preventDefault();
    setContextMenuPosition({
      top: event.clientY + 2,
      left: event.clientX + 2,
    });
  };

  return (
    <li>
      <span className="view-trigger-group-name" onContextMenu={openContextMenu}>
        {group.name}
      </span>
      {!!group.children.length && (
        <ul className="view-trigger-group-sublist">
          {group.children.map(({ variant, value: id }) => {
            if (variant === 'T') {
              return <TriggerListItem key={id} triggerID={id} />;
            } else if (variant === 'G') {
              return <TriggerGroupListItem key={id} groupID={id} />;
            }
          })}
        </ul>
      )}
      {contextMenuPosition && (
        <TriggerGroupContextMenu
          triggerGroup={group}
          onEdit={() => setEditDialogState('edit')}
          onInsertGroup={(offset) => {
            const thisPosition = $positionOf({ group: group.id })(
              store.getState()
            );
            setEditDialogState({
              position: offset + thisPosition,
            });
          }}
          onInsertTrigger={(offset) => {
            const state = store.getState();
            const thisPosition = $positionOf({ group: group.id })(state);
            const ancestorGroups = $groupsUptoGroup(group.parent_id)(state);
            dispatch(
              editNewTrigger({
                parentID: group.parent_id,
                parentPosition: thisPosition + offset,
                ancestorGroups,
              })
            );
          }}
          onDelete={async () => {
            dispatch(applyDeltas(await deleteTriggerGroup(group.id)));
          }}
          close={() => setContextMenuPosition(null)}
          {...contextMenuPosition}
        />
      )}
      {editDialogState !== 'closed' && (
        <TriggerGroupEditorDialog
          {...(editDialogState === 'edit'
            ? { name: group.name, comment: group.comment }
            : { name: '', comment: null })}
          onSave={async (name, comment) => {
            const deltas =
              editDialogState === 'edit'
                ? saveTriggerGroup(groupID, name, comment)
                : createTriggerGroup(
                    name,
                    comment,
                    group.parent_id,
                    editDialogState.position
                  );
            dispatch(applyDeltas(await deltas));
          }}
          close={() => setEditDialogState('closed')}
        />
      )}
    </li>
  );
};

export default React.memo(TriggerGroupListItem);
