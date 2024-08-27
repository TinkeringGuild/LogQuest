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
import { TimerLifetime } from '../generated/TimerLifetime';
import { TimerStateUpdate } from '../generated/TimerStateUpdate';
import { startSync } from '../ipc';
import { println } from '../util';
import Countdown from './Countdown';
import DynamicContainer from './DynamicContainer';

import '../base.css';
import './OverlayWindow.css';

function OverlayWindow() {
  const [editable, setEditable] = useState(false);
  const dispatch = useDispatch();

  useEffect(() => {
    const unlisten = listen<TimerStateUpdate>(
      OVERLAY_STATE_UPDATE_EVENT_NAME,
      ({ payload: update }) => {
        println('GOT OVERLAY TIMER STATE UPDATE: ' + JSON.stringify(update));
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
    startSync().then((timerLifetimes) => {
      dispatch(initTimers(timerLifetimes));
    });
  }, [dispatch]);

  const timerLifetimes: TimerLifetime[] = useSelector($timers);

  return (
    <div className={`overlay ${editable ? 'is-editable' : 'is-static'}`}>
      <DynamicContainer width={300} height={500} x={0} y={0}>
        {timerLifetimes.map(({ id, name, start_time, end_time, is_hidden }) => (
          <Countdown
            label={name}
            startTime={start_time}
            endTime={end_time}
            isHidden={is_hidden}
            key={id}
          />
        ))}
      </DynamicContainer>
    </div>
  );
}

export default OverlayWindow;
