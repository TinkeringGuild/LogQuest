import { open as openDialog } from '@tauri-apps/api/dialog';

export default async function selectExecutableFile() {
  const filePath = await openDialog(openDialogOptions);
  return filePath as string | null;
}

const openDialogOptions = {
  title: 'Select an executable file',
  directory: false,
  multiple: false,
};
