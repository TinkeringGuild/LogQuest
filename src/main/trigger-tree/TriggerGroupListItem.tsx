import React, { useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { editNewTrigger } from '../../features/triggers/triggerEditorSlice';
import {
  $filter,
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
import EmojiText from '../../widgets/EmojiText';
import TriggerGroupEditorDialog from './dialogs/TriggerGroupEditorDialog';
import TriggerGroupContextMenu from './menus/TriggerGroupContextMenu';
import TriggerListItem from './TriggerListItem';

type GroupEditorCreateState = { parentID: UUID | null; parentPosition: number };

const TriggerGroupListItem: React.FC<{
  groupID: UUID;
}> = ({ groupID }) => {
  const dispatch = useDispatch();
  const group = useSelector($triggerGroup(groupID));

  const filter = useSelector($filter);

  const children = filter?.text.trim()
    ? group.children.filter((tgd) =>
        tgd.variant === 'T'
          ? filter.triggerIDs.has(tgd.value)
          : filter.groupIDs.has(tgd.value)
      )
    : group.children;

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
    <li
      className={
        contextMenuPosition
          ? 'view-trigger-group-list-item-context-menu-open'
          : ''
      }
    >
      <span
        className="view-trigger-group-list-item-name"
        onContextMenu={openContextMenu}
      >
        <EmojiText text={group.name} />
      </span>
      {!!children.length && (
        <ul className="view-trigger-group-sublist">
          {children.map(({ variant, value: id }) => {
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
            const [parentID, parentPosition] =
              offset === 'inside'
                ? [group.id, 0]
                : [
                    group.parent_id,
                    offset + $positionOf({ group: group.id })(store.getState()),
                  ];
            setEditDialogState({
              parentID,
              parentPosition,
            });
          }}
          onInsertTrigger={(offset) => {
            const state = store.getState();
            const [parentID, parentPosition] =
              offset === 'inside'
                ? [group.id, 0]
                : [
                    group.parent_id,
                    offset + $positionOf({ group: group.id })(state),
                  ];
            const ancestorGroups = $groupsUptoGroup(parentID)(state);
            dispatch(
              editNewTrigger({
                parentID,
                parentPosition,
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
                    editDialogState.parentID,
                    editDialogState.parentPosition
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
