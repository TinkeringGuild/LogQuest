import { invoke } from "@tauri-apps/api/tauri";
import { eprintln } from "./util";
import { AppConfig } from "./types";

export async function getConfig(): Promise<AppConfig> {
    return await invokeAppConfigChanger("get_config");
}

export async function setEverQuestDirectory(
    newDir: string,
): Promise<AppConfig> {
    return await invokeAppConfigChanger("set_everquest_dir", { newDir });
}

export async function importGinaTriggersFile(
    filePath: string,
): Promise<AppConfig> {
    return await invokeAppConfigChanger("import_gina_triggers_file", {
        filePath,
    });
}

async function invokeAppConfigChanger(
    rpcName: string,
    params?: any,
): Promise<AppConfig> {
    try {
        if (params) {
            return await invoke(rpcName, params);
        } else {
            return await invoke<AppConfig>(rpcName);
        }
    } catch (err) {
        eprintln("GOT ERROR WITH INVOKE: " + JSON.stringify(err));
        throw err;
    }
}
