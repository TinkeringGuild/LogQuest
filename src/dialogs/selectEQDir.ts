import { open as openDialog } from '@tauri-apps/api/dialog';
import { isString } from 'lodash';

import { MainDispatch } from '../MainStore';
import { enterLoadingState, exitLoadingState } from '../features/app/appSlice';
import { initConfig } from '../features/config/configSlice';
import { LogQuestConfig } from '../generated/LogQuestConfig';
import { setEverQuestDirectory } from '../ipc';
import showErrorMessageAlert from './errorMessage';

export default async function openEQFolderSelectionDialog(
  dispatch: MainDispatch
) {
  const selectedDir = await openDialog({
    title: 'Select your EverQuest installation folder',
    directory: true,
    multiple: false,
  });
  if (isString(selectedDir)) {
    dispatch(enterLoadingState());
    try {
      const promise = setEverQuestDirectory(selectedDir);
      promise.finally(() => dispatch(exitLoadingState()));
      const config: LogQuestConfig = await promise;
      dispatch(initConfig(config));
    } catch (err) {
      showErrorMessageAlert(err as string);
    }
  }
}
