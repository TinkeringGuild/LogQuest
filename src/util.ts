import { invoke } from '@tauri-apps/api/tauri';
import { formatRFC3339 } from 'date-fns/formatRFC3339';

import { ProgressUpdate } from './generated/ProgressUpdate';
import { Timestamp } from './generated/Timestamp';

export function nowTimestamp(): Timestamp {
  // Example: "2024-09-18T19:00:52.234Z"
  return formatRFC3339(new Date(), {
    fractionDigits: 3,
  });
}

export function println(message: any) {
  console.log('PRINTLN:', message);
  invoke('print_to_stdout', { message });
}

export function eprintln(message: any) {
  console.error('EPRINTLN', message);
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
