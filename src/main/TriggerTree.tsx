import DownloadingIcon from '@mui/icons-material/Downloading';
import Button from '@mui/material/Button';
import { CSSProperties } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import openGINATriggerFileDialog from '../dialogs/importGINAFile';
import {
  setTriggerEnabled,
  $triggerGroups,
  activateTriggerID,
  $currentTriggerID,
} from '../features/triggers/triggersSlice';
import { Trigger } from '../generated/Trigger';
import { TriggerGroup } from '../generated/TriggerGroup';
import { TriggerGroupDescendant } from '../generated/TriggerGroupDescendant';
import { setTriggerEnabled as ipcSetTriggerEnabled } from '../ipc';
import { Switch } from '@mui/material';

import './TriggerTree.css';

const TriggerTree: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const triggerGroups: TriggerGroup[] = useSelector($triggerGroups);

  return (
    <div className="trigger-tree">
      <div id="main-scrollable" style={styleMainScrollable}>
        <p style={{ textAlign: 'right' }}>
          <Button
            size="small"
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
    </div>
  );
};

const ViewTrigger: React.FC<{ trigger: Trigger; selected: boolean }> = ({
  trigger,
  selected,
}) => {
  const dispatch = useDispatch();
  return (
    <li
      className={`view-trigger-list-item ${selected ? 'view-trigger-list-item-selected' : ''}`}
    >
      <Switch
        size="small"
        checked={trigger.enabled}
        onChange={({ target: { checked } }) => {
          dispatch(
            setTriggerEnabled({ triggerID: trigger.id, enabled: checked })
          );
          ipcSetTriggerEnabled(trigger.id, checked);
        }}
      />{' '}
      <span onClick={() => dispatch(activateTriggerID(trigger.id))}>
        {trigger.name}
      </span>
    </li>
  );
};

const ViewTriggerGroup: React.FC<{ group: TriggerGroup }> = ({ group }) => {
  const currentTriggerID = useSelector($currentTriggerID);
  return (
    <li>
      {group.name}
      {group.children.length && (
        <ul className="view-trigger-group-sublist">
          {group.children.map((descendant: TriggerGroupDescendant) => {
            if ('T' in descendant) {
              return (
                <ViewTrigger
                  key={descendant.T.id}
                  trigger={descendant.T}
                  selected={descendant.T.id === currentTriggerID}
                />
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
