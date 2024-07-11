import {
  open as openDialog,
  message as messageDialog,
} from "@tauri-apps/api/dialog";
// import { emit } from "@tauri-apps/api/event";
// import { exit as exitProcess } from "@tauri-apps/api/process";

import { useEffect } from "react";
import { useSelector, useDispatch } from "react-redux";
import { isArray } from "lodash";

import {
  selectNeedsSetup,
  selectConfigHasLoaded,
  selectEQDir,
  updateConfig,
  updateEverQuestDirectory,
} from "../features/config/configSlice";
import {
  setEverQuestDirectory,
  importGinaTriggersFile,
  getConfig,
} from "../rpc";

import LoadingIndicator from "../widgets/LoadingIndicator";

import "../style/main.scss";
import "./MainWindow.scss";

function MainWindow() {
  const configHasLoaded = useSelector(selectConfigHasLoaded);
  const appNeedsSetup = useSelector(selectNeedsSetup);
  const dispatch = useDispatch();

  useEffect(() => {
    getConfig().then((config) => dispatch(updateConfig(config)));
  }, []);

  return (
    <div>
      <h1 className="title">LogQuest</h1>
      <div className="container">
        {configHasLoaded ? (
          appNeedsSetup ? (
            <Setup />
          ) : (
            <DefaultView />
          )
        ) : (
          <div className="row">
            <LoadingIndicator />
          </div>
        )}
      </div>
    </div>
  );
}

const DefaultView: React.FC<{}> = () => {
  return (
    <div>
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

  async function openGINATriggerFileDialog() {
    const ginaTriggersFile = await openDialog({
      title: "Import a GINA Triggers Package file",
      directory: false,
      multiple: false,
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
    if (ginaTriggersFile) {
      const filePath: string = isArray(ginaTriggersFile)
        ? ginaTriggersFile[0]
        : ginaTriggersFile;
      importGinaTriggersFile(filePath);
    }
  }
};

const Setup: React.FC<{}> = () => {
  const eqDir = useSelector(selectEQDir);
  const dispatch = useDispatch();
  return (
    <div>
      <h2>Welcome!</h2>
      <p>
        To use LogQuest, select the directory of your EverQuest Titanium
        installation:
      </p>
      {eqDir ? (
        <div>
          <code>{eqDir}</code>
          <button onClick={openEQFolderSelectionDialog}>
            Change Directory
          </button>
        </div>
      ) : (
        <>
          <button onClick={openEQFolderSelectionDialog}>
            Select Installation Directory
          </button>
        </>
      )}
    </div>
  );

  async function openEQFolderSelectionDialog() {
    const selectedDir = await openDialog({
      title: "Select your EverQuest installation folder",
      directory: true,
    });
    if (selectedDir) {
      try {
        const savedDir = await setEverQuestDirectory(selectedDir as string);
        dispatch(updateEverQuestDirectory(savedDir));
      } catch (err) {
        showErrorMessageAlert(err as string);
      }
    }
  }
};

function showErrorMessageAlert(message: string) {
  messageDialog(message, { title: "Error", type: "error" });
}

export default MainWindow;
