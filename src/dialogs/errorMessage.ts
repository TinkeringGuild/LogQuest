import { message as messageDialog } from '@tauri-apps/api/dialog';

export default function showErrorMessageAlert(message: string) {
  messageDialog(message, { title: 'Error!', type: 'error' });
}
