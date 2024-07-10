import { invoke } from "@tauri-apps/api/tauri";

export function println(message: any) {
    invoke("print_to_stdout", { message });
}

export function eprintln(message: any) {
    invoke("print_to_stderr", { message });
}
