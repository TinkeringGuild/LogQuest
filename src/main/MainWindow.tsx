import React, { ReactNode, useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import {
  $isBootstrapped,
  $currentMode,
  $isLoading,
  bootstrapHasLoaded,
} from '../features/app/appSlice';
import { loadingWhile } from '../features/app/loadingWhile';
import { initConfig } from '../features/config/configSlice';
import { initOverlay } from '../features/overlay/overlaySlice';
import { initTriggers } from '../features/triggers/triggersSlice';
import { Bootstrap } from '../generated/Bootstrap';
import { getBootstrap } from '../ipc';
import LoadingIndicator from '../widgets/LoadingIndicator';
import Layout from './Layout';
import TriggerTree from './TriggerTree';

const MainWindow: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const isLoading = useSelector($isLoading);

  useEffect(() => {
    loadingWhile(getBootstrap()).then((b: Bootstrap) => {
      dispatch(initConfig(b.config));
      dispatch(initTriggers(b.triggers));
      dispatch(initOverlay(b.overlay));
      dispatch(bootstrapHasLoaded());
    });
  }, [dispatch]);

  const currentMode = useSelector($currentMode);
  const modeNode: ReactNode = (() => {
    if (isLoading) {
      return <LoadingState />;
    }
    switch (currentMode) {
      case 'overview':
        return <h1>Overview</h1>;
      case 'triggers':
        return <TriggerTree />;
      case 'overlay':
        return <h1>Overlay</h1>;
      case 'help':
        return <h1>Help</h1>;
      case 'about':
        return <h1>About</h1>;
      case 'settings':
        return <h1>Settings</h1>;
      default:
        throw new Error(`UNHANDLED MODE: ${currentMode}`);
    }
  })();

  return <Layout>{modeNode}</Layout>;
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

export default MainWindow;
