import { invoke } from "@tauri-apps/api/tauri";

export function println(message: string) {
    invoke("print_to_console", { message });
}
