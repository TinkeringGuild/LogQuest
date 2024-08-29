import { useSelector } from 'react-redux';

import {
  $overlayEditable,
  $overlayMessages,
  $overlayOpacity,
} from '../features/overlay/overlaySlice';
import { $timers } from '../features/timers/timersSlice';
import { TimerLifetime } from '../generated/TimerLifetime';
import Countdown from './Countdown';
import DynamicContainer from './DynamicContainer';
import OverlayMessage from './OverlayMessage';

import '../base.css';
import './OverlayWindow.css';

function OverlayWindow() {
  const editable = useSelector($overlayEditable);
  const opacity = useSelector($overlayOpacity) / 100;

  const timerLifetimes: TimerLifetime[] = useSelector($timers);
  const messages = useSelector($overlayMessages);

  return (
    <div
      className={`overlay ${editable ? 'is-editable' : 'is-static'}`}
      style={{ opacity }}
    >
      <DynamicContainer width={250} height={500} x={0} y={0}>
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
      <DynamicContainer width={500} height={300} x={350} y={0}>
        {editable && (
          <OverlayMessage text="This shows what overlay messages look like" />
        )}
        {messages.map(({ id, text }) => (
          <OverlayMessage key={id} text={text} />
        ))}
      </DynamicContainer>
    </div>
  );
}

export default OverlayWindow;
