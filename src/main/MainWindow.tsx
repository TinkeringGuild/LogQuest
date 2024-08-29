import { Slide, SlideProps, Snackbar } from '@mui/material';
import { TransitionProps } from '@mui/material/transitions';
import { listen } from '@tauri-apps/api/event';
import { isString } from 'lodash';
import React, { ReactNode, useEffect, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import {
  $currentMode,
  $isLoading,
  $progress,
  bootstrapHasLoaded,
  updateProgress,
  updateProgressFinished,
} from '../features/app/appSlice';
import { loadingWhile } from '../features/app/loadingWhile';
import { initConfig } from '../features/config/configSlice';
import { initOverlay } from '../features/overlay/overlaySlice';
import { initTriggers } from '../features/triggers/triggersSlice';
import { Bootstrap } from '../generated/Bootstrap';
import {
  PROGRESS_UPDATE_EVENT_NAME,
  PROGRESS_UPDATE_FINISHED_EVENT_NAME,
} from '../generated/constants';
import { ProgressUpdate } from '../generated/ProgressUpdate';
import { getBootstrap } from '../ipc';
import LoadingIndicator from '../widgets/LoadingIndicator';
import Layout from './Layout';
import OverlayMode from './OverlayMode';
import TriggerTree from './TriggerTree';

import './MainWindow.css';

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
        return <OverlayMode />;
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

  return (
    <Layout>
      {modeNode}
      <SnackBarNotification />
    </Layout>
  );
};

const LoadingState: React.FC<{}> = () => {
  const progress = useSelector($progress);
  const dispatch = useDispatch();

  useEffect(() => {
    const unlisten = listen<ProgressUpdate>(
      PROGRESS_UPDATE_EVENT_NAME,
      ({ payload: update }) => {
        dispatch(updateProgress(update));
      }
    );
    return () => {
      unlisten.then((fn) => {
        fn();
      });
    };
  }, [dispatch]);

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        position: 'absolute',
        top: 0,
        right: 0,
        bottom: 0,
        left: 0,
      }}
    >
      <div style={{ flex: 1 }}></div>
      <div style={{ flex: 0 }}>
        <LoadingIndicator />
      </div>
      <div
        style={{ flex: 1, alignItems: 'flex-start', justifyContent: 'center' }}
      >
        {progress && <ViewProgressUpdate update={progress} />}
      </div>
    </div>
  );
};

const ViewProgressUpdate: React.FC<{ update: ProgressUpdate }> = ({
  update,
}) => {
  const message: string = (() => {
    if (update === 'Started') {
      return '';
    } else if ('Message' in update) {
      return update.Message.text;
    } else if ('Finished' in update) {
      return update.Finished.text;
    } else {
      return '';
    }
  })();

  if (message === '') {
    return <></>;
  }

  return (
    <div style={{ textAlign: 'center' }}>
      {message.split('\n').map((segment, index) => (
        <p style={{ fontWeight: 'bold', margin: 0, padding: 0 }} key={index}>
          {segment}
        </p>
      ))}
    </div>
  );
};

const SnackBarNotification: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const [message, setMessage] = useState('');

  useEffect(() => {
    const unlisten = listen<ProgressUpdate>(
      PROGRESS_UPDATE_FINISHED_EVENT_NAME,
      ({ payload }) => {
        if (!isString(payload) && 'Finished' in payload) {
          setMessage(payload.Finished.text);
          dispatch(updateProgressFinished());
        }
      }
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [dispatch]);

  const open = message !== '';

  const SlideUp = (props: TransitionProps & SlideProps) => (
    <Slide direction="up" {...props} />
  );

  if (open) {
    return (
      <Snackbar
        open={true}
        autoHideDuration={6000}
        TransitionComponent={SlideUp}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        onClose={() => {
          setMessage('');
        }}
        message={message}
      />
    );
  }
};

export default MainWindow;
