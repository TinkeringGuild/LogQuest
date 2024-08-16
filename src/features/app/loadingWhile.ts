import store from '../../MainStore';
import { enterLoadingState, exitLoadingState } from './appSlice';

export function loadingWhile<P>(promise: Promise<P>): Promise<P> {
  store.dispatch(enterLoadingState());
  promise.finally(() => store.dispatch(exitLoadingState()));
  return promise;
}
