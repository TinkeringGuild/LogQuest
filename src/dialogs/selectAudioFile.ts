import { open as openDialog, OpenDialogOptions } from '@tauri-apps/api/dialog';

const openDialogOptions: OpenDialogOptions = {
  title: 'Select an audio file',
  directory: false,
  multiple: false,
  filters: [
    {
      name: 'Audio files (MP3/OGG/WAV/AAC/FLAC)',
      extensions: ['mp3', 'ogg', 'wav', 'aac', 'flac'],
    },
  ],
};

export default async function selectAudioFileDialog() {
  const filePath = await openDialog(openDialogOptions);
  return filePath as string | null;
}
