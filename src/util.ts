import { invoke } from '@tauri-apps/api/tauri';

export function println(message: any) {
  console.log('PRINTLN: ' + message);
  invoke('print_to_stdout', { message });
}

export function eprintln(message: any) {
  console.error('EPRINTLN: ' + message);
  invoke('print_to_stderr', { message });
}
