import { invoke } from '@tauri-apps/api/tauri';
import { ProgressUpdate } from './generated/ProgressUpdate';

export function println(message: any) {
  console.log('PRINTLN: ' + JSON.stringify(message));
  invoke('print_to_stdout', { message });
}

export function eprintln(message: any) {
  console.error('EPRINTLN: ' + JSON.stringify(message));
  invoke('print_to_stderr', { message });
}

export function seqFromProgressUpdate(update: ProgressUpdate | null) {
  if (update === null) {
    return -1;
  } else if (update === 'Started') {
    return 0;
  } else if ('Message' in update) {
    return update.Message.seq;
  } else if ('Finished' in update) {
    return update.Finished.seq;
  } else {
    throw new Error('UNRECOGNIZED PROGRESS UPDATE: ' + JSON.stringify(update));
  }
}
