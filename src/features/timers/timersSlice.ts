import { createSlice } from '@reduxjs/toolkit';
import { LiveTimer } from '../../generated/LiveTimer';
import { TimerStateUpdate } from '../../generated/TimerStateUpdate';
import { remove } from 'lodash';
import { eprintln } from '../../util';

interface TimersState {
  liveTimers: LiveTimer[];
}

const INITIAL_TIMERS_STATE: TimersState = {
  liveTimers: [],
};

const timersSlice = createSlice({
  name: 'timers',
  initialState: INITIAL_TIMERS_STATE,
  reducers: {
    initTimers(
      state: TimersState,
      { payload: liveTimers }: { payload: LiveTimer[] }
    ) {
      state.liveTimers = liveTimers;
    },
    timerStateUpdate(
      state: TimersState,
      { payload: update }: { payload: TimerStateUpdate }
    ) {
      if ('TimerAdded' in update) {
        state.liveTimers.push(update.TimerAdded);
      } else if ('TimerKilled' in update) {
        remove(state.liveTimers, (timer) => timer.id == update.TimerKilled);
      } else {
        eprintln('UNHANDLED TIMER STATE UPDATE: ' + JSON.stringify(update));
      }
    },
  },
});

export default timersSlice.reducer;

export const { timerStateUpdate, initTimers } = timersSlice.actions;

export const selectLiveTimers = ({ timers }: { timers: TimersState }) =>
  timers.liveTimers;
