import { LogQuestConfig } from './generated/LogQuestConfig';
import { OverlayState } from './generated/OverlayState';
import { TriggerRoot } from './generated/TriggerRoot';

export interface Bootstrap {
  overlay: OverlayState;
  config: LogQuestConfig;
  triggers: TriggerRoot;
}

export interface AppState {
  bootstrapped: boolean;
}
