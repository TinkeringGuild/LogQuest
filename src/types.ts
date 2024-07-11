import { LogQuestConfig } from "./generated/LogQuestConfig";
export type { LogQuestConfig } from "./generated/LogQuestConfig";

export interface ConfigWithMetadata {
    config: LogQuestConfig;
    config_has_loaded: boolean;
}

export interface SpellTimer {
    name: string;
    duration: number;
    uuid: string;
}
