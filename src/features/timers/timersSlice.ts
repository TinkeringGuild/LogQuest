import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { TimerLifetime } from '../../generated/TimerLifetime';
import { TimerStateUpdate } from '../../generated/TimerStateUpdate';
import { remove } from 'lodash';
import { eprintln } from '../../util';

export const TIMERS_SLICE = 'timers';

interface TimersState {
  timerLifetimes: TimerLifetime[];
}

const INITIAL_TIMERS_STATE: TimersState = {
  timerLifetimes: [],
};

const timersSlice = createSlice({
  name: TIMERS_SLICE,
  initialState: INITIAL_TIMERS_STATE,
  reducers: {
    initTimers(
      slice: TimersState,
      { payload: timerLifetimes }: PayloadAction<TimerLifetime[]>
    ) {
      slice.timerLifetimes = timerLifetimes;
    },
    timerStateUpdate(
      slice: TimersState,
      { payload: { variant, value } }: PayloadAction<TimerStateUpdate>
    ) {
      if (variant === 'TimerAdded') {
        slice.timerLifetimes.push(value);
      } else if (variant === 'TimerKilled') {
        remove(slice.timerLifetimes, (timer) => timer.id === value);
      } else if (variant == 'TimerHiddenUpdated') {
        const [timerID, newValue] = value;
        const timerLifetime = slice.timerLifetimes.find((t) => t.id == timerID);
        if (timerLifetime) {
          timerLifetime.is_hidden = newValue;
        }
      } else if (variant === 'TimerRestarted') {
        const timerLifetime = slice.timerLifetimes.find(
          (t) => t.id === value.id
        );
        if (timerLifetime) {
          timerLifetime.start_time = value.start_time;
          timerLifetime.end_time = value.end_time;
        }
      } else {
        eprintln(
          `UNHANDLED TIMER STATE UPDATE ${variant} WITH VALUE: ` +
            JSON.stringify(value)
        );
      }
    },
  },
});
export default timersSlice.reducer;
export const { timerStateUpdate, initTimers } = timersSlice.actions;

export const $timers = ({
  [TIMERS_SLICE]: timers,
}: {
  [TIMERS_SLICE]: TimersState;
}) => timers.timerLifetimes;
