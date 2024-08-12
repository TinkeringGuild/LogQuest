import { Dispatch } from '@reduxjs/toolkit';
import { open as openDialog } from '@tauri-apps/api/dialog';
import { isString } from 'lodash';

import { importGinaTriggersFile } from '../ipc';
import { initTriggers } from '../features/triggers/triggersSlice';

export default async function openGINATriggerFileDialog(dispatch: Dispatch) {
  const ginaTriggersFile = await openDialog(openDialogOptions);
  if (!isString(ginaTriggersFile)) {
    return;
  }
  const trigger_root = await importGinaTriggersFile(ginaTriggersFile);
  dispatch(initTriggers(trigger_root));
}

const openDialogOptions = {
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
