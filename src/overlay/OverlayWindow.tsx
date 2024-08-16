import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import {
  initTimers,
  $timers,
  timerStateUpdate,
} from '../features/timers/timersSlice';
import {
  OVERLAY_EDITABLE_CHANGED_EVENT_NAME,
  OVERLAY_STATE_UPDATE_EVENT_NAME,
} from '../generated/constants';
import { LiveTimer } from '../generated/LiveTimer';
import { TimerStateUpdate } from '../generated/TimerStateUpdate';
import { startSync } from '../ipc';
import { println } from '../util';
import Countdown from './Countdown';
import DynamicContainer from './DynamicContainer';

import './OverlayWindow.css';

function OverlayWindow() {
  const [editable, setEditable] = useState(false);
  const dispatch = useDispatch();

  useEffect(() => {
    const unlisten = listen<TimerStateUpdate>(
      OVERLAY_STATE_UPDATE_EVENT_NAME,
      ({ payload: update }) => {
        println('GOT OVERLAY STATE UPDATE: ' + JSON.stringify(update));
        dispatch(timerStateUpdate(update));
      }
    );
    return () => {
      unlisten.then((f) => f());
    };
  }, [dispatch]);

  useEffect(() => {
    const unlisten = listen<boolean>(
      OVERLAY_EDITABLE_CHANGED_EVENT_NAME,
      ({ payload: newValue }) => {
        setEditable(newValue);
      }
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  useEffect(() => {
    startSync().then((liveTimers) => {
      println('GOT LIVE TIMERS: ' + JSON.stringify(liveTimers));
      dispatch(initTimers(liveTimers));
    });
  }, [dispatch]);

  const liveTimers: LiveTimer[] = useSelector($timers);

  return (
    <div className={`overlay ${editable ? 'is-editable' : 'is-static'}`}>
      <DynamicContainer width={450} height={500} x={0} y={0}>
        {liveTimers.map(({ id, name, duration }) => (
          <Countdown label={name} duration={duration} key={id} />
        ))}
      </DynamicContainer>
    </div>
  );
}

export default OverlayWindow;
