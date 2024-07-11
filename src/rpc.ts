import { invoke } from "@tauri-apps/api/tauri";
import { eprintln } from "./util";
import { LogQuestConfig } from "./types";

export async function getConfig(): Promise<LogQuestConfig> {
    return await invokeConfigChanger("get_config");
}

export async function setEverQuestDirectory(
    newDir: string,
): Promise<LogQuestConfig> {
    return await invokeConfigChanger("set_everquest_dir", { newDir });
}

export async function importGinaTriggersFile(
    filePath: string,
): Promise<LogQuestConfig> {
    return await invokeConfigChanger("import_gina_triggers_file", {
        filePath,
    });
}

async function invokeConfigChanger(
    rpcName: string,
    params?: any,
): Promise<LogQuestConfig> {
    try {
        if (params) {
            return await invoke(rpcName, params);
        } else {
            return await invoke<LogQuestConfig>(rpcName);
        }
    } catch (err) {
        eprintln("GOT ERROR WITH INVOKE: " + JSON.stringify(err));
        throw err;
    }
}
