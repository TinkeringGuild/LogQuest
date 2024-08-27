import React, { useRef } from 'react';
import { parseISO } from 'date-fns/parseISO';
import { differenceInMilliseconds } from 'date-fns';

import { Timestamp } from '../generated/Timestamp';

import './Countdown.css';

interface CountdownProps {
  label: string;
  startTime: Timestamp;
  endTime: Timestamp;
  isHidden: boolean;
}

const Countdown: React.FC<CountdownProps> = ({
  label,
  startTime,
  endTime,
  isHidden,
}) => {
  const animatedRemainingRef = useRef<HTMLDivElement | null>(null);
  const animatedWarningRef = useRef<HTMLDivElement | null>(null);

  const parsedStartTime: Date = parseISO(startTime);
  const parsedEndTime: Date = parseISO(endTime);
  const durationMillis = differenceInMilliseconds(
    parsedEndTime,
    parsedStartTime
  );

  const now = new Date();
  const elapsedMillisSinceStart = Math.max(
    0,
    differenceInMilliseconds(now, parsedStartTime)
  );

  const progressPercent = (elapsedMillisSinceStart * 100) / durationMillis;
  const percentRemaining = 100 - progressPercent;
  const remainingDurationMillis = durationMillis * (percentRemaining / 100);

  restartAnimatedClass(
    'countdown-animation-remaining',
    () => animatedRemainingRef.current
  );
  restartAnimatedClass(
    'countdown-animation-warning',
    () => animatedWarningRef.current
  );

  const cssVars = {
    '--start-percent': `${percentRemaining}%`,
  } as React.CSSProperties;

  return (
    <div
      className="countdown countdown-animation-warning column-member"
      ref={animatedWarningRef}
      style={{
        animationDuration: `${remainingDurationMillis}ms`,
        display: isHidden ? 'none' : 'inherit',
      }}
    >
      {/* This is the element with the shrink animation */}
      <div
        ref={animatedRemainingRef}
        className="countdown-animation-remaining"
        style={{
          animationDuration: `${remainingDurationMillis}ms`,
          ...cssVars,
        }}
      ></div>
      <p>{label}</p>
    </div>
  );
};

function restartAnimatedClass(
  className: string,
  getRef: () => HTMLElement | null
) {
  const el = getRef();
  if (el) {
    el.classList.remove(className);
    void el.offsetWidth; // force reflow
    el.classList.add(className);
  }
}

const CountdownMemoized = React.memo(Countdown);
export default CountdownMemoized;
