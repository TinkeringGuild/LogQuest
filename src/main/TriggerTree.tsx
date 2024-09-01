import DownloadingIcon from '@mui/icons-material/Downloading';
import Button from '@mui/material/Button';
import { CSSProperties } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import openGINATriggerFileDialog from '../dialogs/importGINAFile';
import {
  setTriggerEnabled,
  $triggerGroups,
} from '../features/triggers/triggersSlice';
import { Trigger } from '../generated/Trigger';
import { TriggerGroup } from '../generated/TriggerGroup';
import { TriggerGroupDescendant } from '../generated/TriggerGroupDescendant';
import { setTriggerEnabled as ipcSetTriggerEnabled } from '../ipc';

const TriggerTree: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const triggerGroups: TriggerGroup[] = useSelector($triggerGroups);
  return (
    <div id="main-scrollable" style={styleMainScrollable}>
      <p style={{ textAlign: 'right' }}>
        <Button
          size="large"
          variant="contained"
          startIcon={<DownloadingIcon />}
          onClick={() => openGINATriggerFileDialog(dispatch)}
        >
          Import GINA Export
        </Button>
      </p>
      <div>
        {triggerGroups.length ? (
          <ul>
            {triggerGroups.map((group) => (
              <ViewTriggerGroup key={group.id} group={group} />
            ))}
          </ul>
        ) : (
          <p>You have not created any triggers yet.</p>
        )}
      </div>
    </div>
  );
};

const ViewTrigger: React.FC<{ trigger: Trigger }> = ({ trigger }) => {
  const dispatch = useDispatch();
  return (
    <li>
      <input
        type="checkbox"
        checked={trigger.enabled}
        onChange={({ target: { checked } }) => {
          dispatch(
            setTriggerEnabled({ triggerID: trigger.id, enabled: checked })
          );
          ipcSetTriggerEnabled(trigger.id, checked);
        }}
      />{' '}
      {trigger.name}
    </li>
  );
};

const ViewTriggerGroup: React.FC<{ group: TriggerGroup }> = ({ group }) => {
  return (
    <li>
      {group.name}
      {group.children.length && (
        <ul>
          {group.children.map((descendant: TriggerGroupDescendant) => {
            if ('T' in descendant) {
              return (
                <ViewTrigger key={descendant.T.id} trigger={descendant.T} />
              );
            } else if ('TG' in descendant) {
              return (
                <ViewTriggerGroup
                  key={descendant.TG.id}
                  group={descendant.TG}
                />
              );
            }
          })}
        </ul>
      )}
    </li>
  );
};

const styleMainScrollable: CSSProperties = {
  position: 'absolute',
  top: 0,
  right: 0,
  left: 0,
  bottom: 0,
  overflowY: 'scroll',
  overflowX: 'hidden',
  // scrollbarGutter: 'stable',
};

export default TriggerTree;
