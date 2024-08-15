import { CSSProperties, useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import DownloadingIcon from '@mui/icons-material/Downloading';
import Button from '@mui/material/Button';

import openGINATriggerFileDialog from '../dialogs/importGINAFile';
import { bootstrapHasLoaded, hasBootstrapped } from '../features/app/appSlice';
import { initConfig } from '../features/config/configSlice';
import { initOverlay } from '../features/overlay/overlaySlice';
import {
  initTriggers,
  selectTriggerGroups,
} from '../features/triggers/triggersSlice';
import { Bootstrap } from '../generated/Bootstrap';
import { Trigger } from '../generated/Trigger';
import { TriggerGroup } from '../generated/TriggerGroup';
import { TriggerGroupDescendant } from '../generated/TriggerGroupDescendant';
import { getBootstrap } from '../ipc';
import LoadingIndicator from '../widgets/LoadingIndicator';

const TriggerTree: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const bootstrapped = useSelector(hasBootstrapped);

  useEffect(() => {
    getBootstrap().then((b: Bootstrap) => {
      dispatch(initConfig(b.config));
      dispatch(initTriggers(b.triggers));
      dispatch(initOverlay(b.overlay));
      dispatch(bootstrapHasLoaded());
    });
  }, []);

  if (!bootstrapped) {
    return <LoadingState />;
  }

  return <TreeView />;
};

const TreeView: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const triggerGroups: TriggerGroup[] = useSelector(selectTriggerGroups);
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
              <ViewTriggerGroup group={group} />
            ))}
          </ul>
        ) : (
          <p>You have not created any triggers yet.</p>
        )}
      </div>
    </div>
  );
};

const ViewTrigger: React.FC<{ trigger: Trigger }> = ({ trigger }) => (
  <li>
    <input type="checkbox" checked={trigger.enabled} /> {trigger.name}
  </li>
);

const ViewTriggerGroup: React.FC<{ group: TriggerGroup }> = ({ group }) => {
  return (
    <li key={group.id}>
      {group.name}
      {group.children.length > 0 && (
        <ul>
          {group.children.map((descendant: TriggerGroupDescendant) => {
            if ('T' in descendant) {
              return <ViewTrigger trigger={descendant.T} />;
            } else if ('TG' in descendant) {
              return <ViewTriggerGroup group={descendant.TG} />;
            }
          })}
        </ul>
      )}
    </li>
  );
};

const LoadingState: React.FC<{}> = () => (
  <div
    style={{
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      position: 'absolute',
      top: 0,
      right: 0,
      bottom: 0,
      left: 0,
    }}
  >
    <LoadingIndicator />
  </div>
);

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
