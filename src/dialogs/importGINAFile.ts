import { open as openDialog, OpenDialogOptions } from '@tauri-apps/api/dialog';
import { isString } from 'lodash';

import { loadingWhile } from '../features/app/loadingWhile';
import { initTriggers } from '../features/triggers/triggersSlice';
import { importGinaTriggersFile } from '../ipc';
import { MainDispatch } from '../MainStore';

export default async function openGINATriggerFileDialog(
  dispatch: MainDispatch
) {
  const ginaTriggersFile = await openDialog(openDialogOptions);
  if (!isString(ginaTriggersFile)) {
    return;
  }
  const trigger_root = await loadingWhile(
    importGinaTriggersFile(ginaTriggersFile)
  );
  dispatch(initTriggers(trigger_root));
}

const openDialogOptions: OpenDialogOptions = {
  title: 'Import a GINA Triggers Package file',
  directory: false,
  multiple: false,
  filters: [
    {
      name: 'GINA Triggers Package (.gtp) file',
      extensions: ['gtp'],
    },
    {
      name: 'GINA Triggers Package SharedData.xml file',
      extensions: ['xml'],
    },
  ],
};
