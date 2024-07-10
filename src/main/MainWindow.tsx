/* import { exit as exitProcess } from "@tauri-apps/api/process"; */
import {
  open as openDialog,
  /* message as messageDialog, */
} from "@tauri-apps/api/dialog";
/* import { emit } from "@tauri-apps/api/event"; */

/* import { useEffect, useContext } from "react"; */
import { useSelector, useDispatch } from "react-redux";

import {
  selectEQDir,
  updateEverQuestDirectory,
} from "../features/config/configSlice";
import { setEverQuestDirectory } from "../rpc";
/* import { AppConfig } from "../types"; */
/* import ReceiveBuffs from "./ReceiveBuffs"; */
/* import LoadingIndicator from "../widgets/LoadingIndicator"; */

import "../base.css";
import "./MainWindow.css";

function MainWindow() {
  const eqDir = useSelector(selectEQDir);
  const dispatch = useDispatch();
  return (
    <div className="container">
      {true && (
        <div>
          <h3>Select your EverQuest installation folder</h3>
          <p>
            <input type="text" value={eqDir} />
            <button onClick={openEQFolderSelectionDialog}>Select Folder</button>
          </p>
        </div>
      )}
      <div>
        <h3>Create new Triggers</h3>
        <button>Edit Triggers</button>
      </div>
      <div>
        <h3>Import a GINA trigger package</h3>
        <button onClick={openGINATriggerFileDialog}>Import file</button>
      </div>
      {/* <ReceiveBuffs /> */}
    </div>
  );

  async function openEQFolderSelectionDialog() {
    const selectedDir = await openDialog({
      title: "Select your EverQuest installation folder",
      directory: true,
    });
    if (selectedDir) {
      const savedDir = await setEverQuestDirectory(selectedDir as string);
      dispatch(updateEverQuestDirectory(savedDir));
    }
  }
}

function openGINATriggerFileDialog() {
  return openDialog({
    title: "Import a GINA Triggers Package file",
    directory: false,
    filters: [
      {
        name: "GINA Triggers Package (.gtp) file",
        extensions: ["gtp"],
      },
      {
        name: "GINA Triggers Package SharedData.xml file",
        extensions: ["xml"],
      },
    ],
  });
}

export default MainWindow;

/*
function Initializer() {
  const { appConfig, setAppConfig } = useContext(AppConfigContext);

  useEffect(() => {
    invoke<AppConfig>("get_config")
      .then(setAppConfig)
      .catch(() => {
        // setTimeout is needed to prevent the app from locking up
        setTimeout(showFatalErrorAlert, 0);
      });
  }, []);

  if (appConfig === null) {
    return (
      <div className="container">
        <div className="centered-content">
          <LoadingIndicator />
        </div>
      </div>
    );
  }

  return (
    <AppConfigProvider>
      <MainWindow />
    </AppConfigProvider>
  );
}

function showFatalErrorAlert() {
  messageDialog(`LogQuest experienced a fatal error!\n\n${err}`, {
    title: "FATAL ERROR",
    okLabel: "Terminate",
    type: "error",
  }).finally(() => exitProcess(10));
}

export default Initializer;
*/

/* interface InitWithConfigAction {
 *   type: "InitWithConfig";
 *   config: AppConfig;
 * }
 *
 * function reducer(state: MainState, action: MainAction): MainState {
 *   switch (action.type) {
 *     case "InitWithConfig":
 *       return {
 *         ...state,
 *         app_config: action.config,
 *       };
 *   }
 * } */
