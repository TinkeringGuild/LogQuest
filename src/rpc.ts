import { invoke } from "@tauri-apps/api/tauri";
import { eprintln } from "./util";

export async function setEverQuestDirectory(newDir: string): Promise<string> {
    try {
        const response: string = await invoke("set_everquest_dir", { newDir });
        return response;
    } catch (err) {
        eprintln("GOT ERROR WITH INVOKE: " + JSON.stringify(err));
        throw err;
    }
}
