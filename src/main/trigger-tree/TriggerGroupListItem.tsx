import React, { useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import {
  $triggerGroup,
  applyDeltas,
} from '../../features/triggers/triggersSlice';
import { UUID } from '../../generated/UUID';
import TriggerGroupEditorDialog from './dialogs/TriggerGroupEditorDialog';
import TriggerListItem from './TriggerListItem';
import TriggerGroupContextMenu from './menus/TriggerGroupContextMenu';
import { deleteTriggerGroup } from '../../ipc';

const TriggerGroupListItem: React.FC<{
  groupID: UUID;
}> = ({ groupID }) => {
  const dispatch = useDispatch();
  const group = useSelector($triggerGroup(groupID));

  const [editDialogIsOpen, setEditDialogIsOpen] = useState(false);

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
          onEdit={() => setEditDialogIsOpen(true)}
          onDelete={async () =>
            dispatch(applyDeltas(await deleteTriggerGroup(group.id)))
          }
          close={() => setContextMenuPosition(null)}
          {...contextMenuPosition}
        />
      )}
      {editDialogIsOpen && (
        <TriggerGroupEditorDialog
          triggerGroup={group}
          close={() => setEditDialogIsOpen(false)}
        />
      )}
    </li>
  );
};

export default React.memo(TriggerGroupListItem);
