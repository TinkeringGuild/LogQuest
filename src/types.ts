export interface AppConfig {
    everquest_directory: string;
}

export interface ConfigWithMetadata {
    config: AppConfig;
    config_has_loaded: boolean;
}

export interface SpellTimer {
    name: string;
    duration: number;
    uuid: string;
}
