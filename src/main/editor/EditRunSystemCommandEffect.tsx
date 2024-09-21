import { debounce } from 'lodash';
import { useEffect, useId, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import FileOpenOutlined from '@mui/icons-material/FileOpenOutlined';
import Checkbox from '@mui/material/Checkbox';
import FormControlLabel from '@mui/material/FormControlLabel';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';
import StandardTooltip from '../../widgets/StandardTooltip';

import selectExecutableFile from '../../dialogs/selectExecutable';
import {
  $errorForID,
  EffectVariantRunSystemCommand,
  forgetError,
  setCommandTemplateSecurityCheck,
  setError,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import { CommandTemplate } from '../../generated/CommandTemplate';
import { CommandTemplateSecurityCheck } from '../../generated/CommandTemplateSecurityCheck';
import { SystemCommandInfo } from '../../generated/SystemCommandInfo';
import { getSystemCommandInfo, signCommandTemplate } from '../../ipc';
import EffectWithOptions from './EffectWithOptions';
import ControlledTextField from './widgets/ControlledTextField';

// TODO: Command arguments should be treated as an Array, not split by whitespace
// TODO: Check if crypto is enabled. Should be a global static config thing
// TODO: Allow specifying environment variables for the command

const getSystemCommandInfoDebounced = debounce(
  (command: string, thenFn: (sysCmdInfo: SystemCommandInfo) => void) => {
    getSystemCommandInfo(command).then(thenFn);
  },
  300
);

const EditRunSystemCommandEffect: React.FC<{
  selector: TriggerEditorSelector<EffectVariantRunSystemCommand>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const dispatch = useDispatch();
  const fromStore = useSelector(triggerEditorSelector(selector));

  const commandInputID = useId();

  const [commandInput, setCommandInput] = useState('');
  const [paramsInput, setParamsInput] = useState('');
  const [stdinInput, setStdinInput] = useState<string | null>(null);

  const [commandPath, setCommandPath] = useState<string | undefined>(undefined);

  const [commandFileSelectDialogOpen, setCommandFileSelectDialogOpen] =
    useState(false);

  const commandError: string | undefined = useSelector(
    $errorForID(commandInputID)
  );

  // Initializes component state from the store
  useEffect(() => {
    const { command, params, write_to_stdin }: CommandTemplate =
      fromStore.value.variant === 'Approved'
        ? fromStore.value.value[1]
        : fromStore.value.value;

    setCommandInput(command);
    setParamsInput(params.join(' '));
    setStdinInput(write_to_stdin);
  }, []);

  // Handles validation/error of commandInput and manages setCommandPath
  useEffect(() => {
    let isMounted = true; // needed due to async logic

    if (!commandInput.trim()) {
      dispatch(
        setError({ id: commandInputID, error: 'Command cannot be blank' })
      );
    } else {
      dispatch(forgetError(commandInputID));
      getSystemCommandInfoDebounced(
        commandInput,
        async (cmdInfo: SystemCommandInfo) => {
          if (!isMounted) {
            return;
          }
          const errorState = validateSystemCommandInfo(commandInput, cmdInfo);
          const [hasError, helperText] = errorState;
          if (hasError) {
            setCommandPath(undefined);
            dispatch(setError({ id: commandInputID, error: helperText }));
          } else {
            setCommandPath(helperText);
            dispatch(forgetError(commandInputID));
          }
        }
      );
    }
    return () => {
      isMounted = false;
    };
  }, [commandInput, paramsInput, stdinInput, commandInputID]);

  // Cleanup error if component is unmounted
  useEffect(
    () => () => {
      dispatch(forgetError(commandInputID));
    },
    []
  );

  const createCommandTemplate = () => {
    const trimmedParams = paramsInput.trim();
    return {
      command: commandInput,
      params: trimmedParams ? trimmedParams.split(/\s+/) : [],
      write_to_stdin: stdinInput,
    };
  };

  const setStoreStateAsUnapprovedCommandTemplate = () => {
    const cmdTmpl: CommandTemplate = createCommandTemplate();
    const unapproved: CommandTemplateSecurityCheck = {
      variant: 'Unapproved',
      value: cmdTmpl,
    };
    dispatch(
      setCommandTemplateSecurityCheck({
        selector,
        cmdTmplSecCheck: unapproved,
      })
    );
  };

  const setStoreStateAsApprovedCommandTemplate = async () => {
    const cmdTmpl: CommandTemplate = createCommandTemplate();
    const cmdTmplSecCheck = await signCommandTemplate(cmdTmpl);
    dispatch(
      setCommandTemplateSecurityCheck({
        selector,
        cmdTmplSecCheck,
      })
    );
  };

  const isApproved = fromStore.value.variant === 'Approved';
  const isValid = !commandError;

  // Synchronizes the store with the component state, preserving Approved status
  useEffect(() => {
    if (!isValid) {
      return;
    }
    if (isApproved) {
      setStoreStateAsApprovedCommandTemplate();
    } else {
      setStoreStateAsUnapprovedCommandTemplate();
    }
  }, [isValid, commandInput, paramsInput, stdinInput, dispatch]);

  const openCommandFileSelectDialog = async () => {
    setCommandFileSelectDialogOpen(true);
    const filePath = await selectExecutableFile();
    setCommandFileSelectDialogOpen(false);
    if (filePath) {
      setCommandInput(filePath);
    }
  };

  return (
    <EffectWithOptions
      variant="RunSystemCommand"
      help="Executes a specific command on your system. This effect will not finish until the command executed finishes."
      onDelete={onDelete}
    >
      <Stack gap={1.5}>
        {/* TODO: When the is_crypto_enabled check is supported client-side and it's not enabled
            this checkbox should be replaced with a notice that commands are not available. */}
        <FormControlLabel
          label="Approve this Command to run on your system"
          control={
            <Checkbox
              checked={isApproved}
              disabled={!isValid}
              indeterminate={!isValid && isApproved}
              color={isApproved ? 'success' : 'warning'}
              onChange={(e) => {
                if (e.target.checked) {
                  setStoreStateAsApprovedCommandTemplate();
                } else {
                  setStoreStateAsUnapprovedCommandTemplate();
                }
              }}
              sx={
                isApproved
                  ? {}
                  : {
                      color: '#ed6c02', // warning color
                    }
              }
            />
          }
        />
        <TextField
          id={commandInputID}
          label="Command"
          fullWidth
          value={commandInput}
          disabled={commandFileSelectDialogOpen}
          error={!!commandError}
          color={commandError ? 'error' : isApproved ? 'success' : 'warning'}
          helperText={
            commandError ||
            (commandPath
              ? `${commandPath} ${isApproved ? '' : ' (NOT APPROVED)'}`
              : 'Detecting command...') // commandPath can be undefined while typing
          }
          focused={!!commandError || !!commandPath}
          onChange={(e) => setCommandInput(e.target.value)}
          onKeyDown={() =>
            commandPath !== undefined && setCommandPath(undefined)
          }
          className="template-input"
          slotProps={{
            input: {
              endAdornment: (
                <InputAdornment position="end">
                  <StandardTooltip help="Select the executable file">
                    <IconButton
                      onClick={openCommandFileSelectDialog}
                      edge="end"
                    >
                      <FileOpenOutlined />
                    </IconButton>
                  </StandardTooltip>
                </InputAdornment>
              ),
            },
          }}
        />
        <ControlledTextField
          label="Command Arguments"
          value={paramsInput}
          onCommit={(input) => setParamsInput(input)}
          className="template-input"
          fullWidth
        />
        <div>
          <FormControlLabel
            label="Send initial input to the process (on STDIN)"
            control={
              <Checkbox
                checked={stdinInput !== null}
                onClick={() => setStdinInput(stdinInput === null ? '' : null)}
              />
            }
          />
          {stdinInput !== null && (
            <ControlledTextField
              fullWidth
              multiline
              label="Written to STDIN (Template)"
              value={stdinInput}
              onCommit={(input) => setStdinInput(input)}
              className="template-input"
              sx={{ mt: 1 }}
            />
          )}
        </div>
      </Stack>
    </EffectWithOptions>
  );
};

function validateSystemCommandInfo(
  command: string,
  cmdInfo: SystemCommandInfo
): [boolean, string] {
  if (cmdInfo.variant === 'Executable') {
    return [false, cmdInfo.value];
  }
  if (cmdInfo.variant === 'NotExecutable') {
    return [true, 'This file is not executable'];
  }
  return [true, `Cannot detect command "${command}" on your system`]; // variant === 'NotFound'
}

export default EditRunSystemCommandEffect;
