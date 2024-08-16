import { createSlice } from '@reduxjs/toolkit';
import { LiveTimer } from '../../generated/LiveTimer';
import { TimerStateUpdate } from '../../generated/TimerStateUpdate';
import { remove } from 'lodash';
import { eprintln } from '../../util';

export const TIMERS_SLICE = 'timers';

interface TimersState {
  liveTimers: LiveTimer[];
}

const INITIAL_TIMERS_STATE: TimersState = {
  liveTimers: [],
};

const timersSlice = createSlice({
  name: TIMERS_SLICE,
  initialState: INITIAL_TIMERS_STATE,
  reducers: {
    initTimers(
      state: TimersState,
      { payload: liveTimers }: { payload: LiveTimer[] }
    ) {
      state.liveTimers = liveTimers;
    },
    timerStateUpdate(
      slice: TimersState,
      { payload: update }: { payload: TimerStateUpdate }
    ) {
      if ('TimerAdded' in update) {
        slice.liveTimers.push(update.TimerAdded);
      } else if ('TimerKilled' in update) {
        remove(slice.liveTimers, (timer) => timer.id == update.TimerKilled);
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
}) => timers.liveTimers;
