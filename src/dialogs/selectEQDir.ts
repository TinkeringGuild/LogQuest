import { Dispatch } from '@reduxjs/toolkit';
import { open as openDialog } from '@tauri-apps/api/dialog';
import { isString } from 'lodash';

import { initConfig } from '../features/config/configSlice';
import { setEverQuestDirectory } from '../ipc';
import showErrorMessageAlert from './errorMessage';

export default async function openEQFolderSelectionDialog(dispatch: Dispatch) {
  const selectedDir = await openDialog({
    title: 'Select your EverQuest installation folder',
    directory: true,
    multiple: false,
  });
  if (isString(selectedDir)) {
    try {
      const config = await setEverQuestDirectory(selectedDir);
      dispatch(initConfig(config));
    } catch (err) {
      showErrorMessageAlert(err as string);
    }
  }
}
