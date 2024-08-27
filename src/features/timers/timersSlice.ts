import { createSlice } from '@reduxjs/toolkit';
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
      state: TimersState,
      { payload: timerLifetimes }: { payload: TimerLifetime[] }
    ) {
      state.timerLifetimes = timerLifetimes;
    },
    timerStateUpdate(
      slice: TimersState,
      { payload: update }: { payload: TimerStateUpdate }
    ) {
      if ('TimerAdded' in update) {
        slice.timerLifetimes.push(update.TimerAdded);
      } else if ('TimerKilled' in update) {
        remove(slice.timerLifetimes, (timer) => timer.id == update.TimerKilled);
      } else {
        eprintln('UNHANDLED TIMER STATE UPDATE: ' + JSON.stringify(update));
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
