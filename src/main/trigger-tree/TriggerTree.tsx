import { os } from '@tauri-apps/api';
import React, { useEffect, useId, useRef, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import Add from '@mui/icons-material/Add';
import AddCircleOutline from '@mui/icons-material/AddCircleOutline';
import ChecklistSharp from '@mui/icons-material/ChecklistSharp';
import Close from '@mui/icons-material/Close';
import DownloadingIcon from '@mui/icons-material/Downloading';
import ManageSearch from '@mui/icons-material/ManageSearch';
import MoreVert from '@mui/icons-material/MoreVert';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';
import ListItemIcon from '@mui/material/ListItemIcon';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import Slide from '@mui/material/Slide';
import TextField from '@mui/material/TextField';
import ToggleButton from '@mui/material/ToggleButton';
import PopupState, { bindMenu, bindTrigger } from 'material-ui-popup-state';

import openGINATriggerFileDialog from '../../dialogs/importGINAFile';
import { editNewTrigger } from '../../features/triggers/triggerEditorSlice';
import {
  $activeTriggerTag,
  $filter,
  $topLevel,
  activateTriggerTagID,
  applyDeltas,
  clearSearch,
  search,
} from '../../features/triggers/triggersSlice';
import { TriggerGroupDescendant } from '../../generated/TriggerGroupDescendant';
import {
  createTriggerGroup,
  createTriggerTag,
  setTriggerTagActivated,
} from '../../ipc';
import StandardTooltip from '../../widgets/StandardTooltip';
import TriggerGroupListItem from './TriggerGroupListItem';
import TriggerIDsInSelectedTriggerTagContext from './TriggerIDsInSelectedTriggerTagContext';
import TriggerListItem from './TriggerListItem';
import TriggerTagChanger from './TriggerTagChanger';

import './TriggerTree.css';
import TriggerGroupEditorDialog from './dialogs/TriggerGroupEditorDialog';
import { updateActivedTriggerTagIDs } from '../../features/app/appSlice';

const TriggerTree: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const triggerTreeMoreMenuID = useId();
  const top: TriggerGroupDescendant[] = useSelector($topLevel);
  const activeTriggerTag = useSelector($activeTriggerTag);

  const [shouldFocusFilter, setShouldFocusFilter] = useState(false);
  const filterInputRef = useRef<HTMLInputElement>(null);
  const filter = useSelector($filter);

  useEffect(() => {
    if (shouldFocusFilter && filterInputRef.current) {
      filterInputRef.current.focus();
      setShouldFocusFilter(false);
    }
  }, [shouldFocusFilter, filterInputRef.current]);

  const activeTriggersSet = activeTriggerTag
    ? {
        tagID: activeTriggerTag.id,
        triggerIDs: new Set(activeTriggerTag.triggers),
      }
    : null;

  const topFiltered = filter?.text.trim()
    ? top.filter((tgd) =>
        tgd.variant === 'T'
          ? filter.triggerIDs.has(tgd.value)
          : filter.groupIDs.has(tgd.value)
      )
    : top;

  return (
    <div className="trigger-tree scrollable-container">
      <div className="scrollable-content central-content">
        <SearchShortcutListener
          onTrigger={() => {
            if (filter) {
              dispatch(clearSearch());
            } else {
              setShouldFocusFilter(true);
              dispatch(search(''));
            }
          }}
        />
        {filter && (
          <Slide
            direction="down"
            timeout={250}
            in={!!filter}
            mountOnEnter
            unmountOnExit
          >
            <div style={{ marginBottom: 20 }}>
              <TextField
                label="Search"
                value={filter.text}
                variant="filled"
                fullWidth
                color={
                  filter && topFiltered.length === 0 ? 'warning' : 'primary'
                }
                inputRef={filterInputRef}
                onChange={(e) => dispatch(search(e.target.value))}
                onKeyDown={(e) => {
                  if (e.key === 'Escape') {
                    dispatch(clearSearch());
                  }
                }}
                slotProps={{
                  input: {
                    endAdornment: (
                      <InputAdornment position="end">
                        <IconButton
                          edge="end"
                          onClick={() => dispatch(clearSearch())}
                        >
                          <Close />
                        </IconButton>
                      </InputAdornment>
                    ),
                  },
                }}
              />
            </div>
          </Slide>
        )}
        <Box
          display="flex"
          alignItems="flex-start"
          justifyItems="right"
          flexDirection="row"
          justifyContent="space-between"
        >
          <TriggerTagChanger
            onChange={(tagIDMaybe) =>
              dispatch(activateTriggerTagID(tagIDMaybe))
            }
            onCreate={async (name) => {
              const deltas = await createTriggerTag(name);
              dispatch(applyDeltas(deltas));
              const creation = deltas.find(
                (delta) => delta.variant === 'TriggerTagCreated'
              );
              if (creation) {
                dispatch(activateTriggerTagID(creation.value.id));
                const activeTriggerTags = await setTriggerTagActivated(
                  creation.value.id,
                  true
                );
                dispatch(updateActivedTriggerTagIDs(activeTriggerTags));
              }
            }}
          />

          <PopupState variant="popover" popupId={triggerTreeMoreMenuID}>
            {(popupState) => (
              <>
                <div>
                  <StandardTooltip help="Toggle Search" placement="left">
                    <span>
                      {/* the span is needed because if ToggleButton is disabled, it won't generate Tooltip events */}
                      <ToggleButton
                        value="filter"
                        size="small"
                        selected={!!filter}
                        disabled={top.length === 0}
                        onChange={() => {
                          !filter && setShouldFocusFilter(true);
                          dispatch(filter ? clearSearch() : search(''));
                        }}
                        sx={{ color: 'black' }}
                      >
                        <ManageSearch />
                      </ToggleButton>
                    </span>
                  </StandardTooltip>{' '}
                  <Button
                    {...bindTrigger(popupState)}
                    className="trigger-tree-more-menu"
                    variant="contained"
                    startIcon={<MoreVert />}
                  >
                    Actions
                  </Button>
                </div>
                <Menu
                  {...bindMenu(popupState)}
                  transformOrigin={{
                    vertical: 'top',
                    horizontal: 'right',
                  }}
                  anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                >
                  <MenuItem onClick={popupState.close} disabled={true}>
                    <ListItemIcon>
                      <ChecklistSharp />
                    </ListItemIcon>
                    Select/Move Mode
                  </MenuItem>
                  <MenuItem
                    onClick={() => {
                      popupState.close();
                      openGINATriggerFileDialog(dispatch);
                    }}
                  >
                    <ListItemIcon>
                      <DownloadingIcon />
                    </ListItemIcon>
                    Import GINA Export
                  </MenuItem>
                </Menu>
              </>
            )}
          </PopupState>
        </Box>

        <TriggerIDsInSelectedTriggerTagContext.Provider
          value={activeTriggersSet}
        >
          <div>
            {topFiltered.length ? (
              <ul>
                {topFiltered.map((tgd) =>
                  tgd.variant === 'T' ? (
                    <TriggerListItem key={tgd.value} triggerID={tgd.value} />
                  ) : (
                    <TriggerGroupListItem key={tgd.value} groupID={tgd.value} />
                  )
                )}
              </ul>
            ) : filter ? (
              <Alert severity="warning" sx={{ mt: 2 }}>
                Nothing matches your search.
              </Alert>
            ) : (
              <div style={{ marginTop: 15 }}>
                <Alert severity="info">
                  You have not created any Triggers yet.
                </Alert>
                <div style={{ marginTop: 15 }}>
                  <Button
                    variant="contained"
                    startIcon={<Add />}
                    onClick={() => {
                      dispatch(
                        editNewTrigger({
                          parentID: null,
                          parentPosition: 0,
                          ancestorGroups: [],
                        })
                      );
                    }}
                  >
                    Create Trigger
                  </Button>{' '}
                  <CreateTriggerGroupButton />
                </div>
              </div>
            )}
          </div>
        </TriggerIDsInSelectedTriggerTagContext.Provider>
      </div>
    </div>
  );
};

const CreateTriggerGroupButton: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const [dialogOpen, setDialogOpen] = useState(false);
  return (
    <>
      <Button
        variant="contained"
        startIcon={<AddCircleOutline />}
        onClick={() => setDialogOpen(true)}
      >
        Create Trigger Group
      </Button>
      {dialogOpen && (
        <TriggerGroupEditorDialog
          name=""
          comment={null}
          onSave={async (name, comment) => {
            const deltas = createTriggerGroup(name, comment, null, 0);
            dispatch(applyDeltas(await deltas));
          }}
          close={() => setDialogOpen(false)}
        />
      )}
    </>
  );
};

const SearchShortcutListener: React.FC<{ onTrigger: () => void }> = ({
  onTrigger,
}) => {
  useEffect(() => {
    let isMounted = true;
    let osType: os.OsType | null = null;

    const osTypePromise = os.type();

    osTypePromise.then((tauriOsType) => {
      if (!isMounted) return;
      osType = tauriOsType;
    });

    const handleKeyDown = (event: KeyboardEvent) => {
      if (!osType) return;

      const modifierKeyIsDown =
        osType === 'Darwin' ? event.metaKey : event.ctrlKey;
      if (modifierKeyIsDown && event.key === 'f') {
        onTrigger();
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    // Clean up the event listener on component unmount
    return () => {
      isMounted = false;
      osTypePromise.then(() => {
        window.removeEventListener('keydown', handleKeyDown);
      });
    };
  }, []);

  return <></>;
};

export default TriggerTree;
