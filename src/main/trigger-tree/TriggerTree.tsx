import React, { useId } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import ChecklistSharp from '@mui/icons-material/ChecklistSharp';
import DownloadingIcon from '@mui/icons-material/Downloading';
import ManageSearch from '@mui/icons-material/ManageSearch';
import MoreVert from '@mui/icons-material/MoreVert';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import PopupState, { bindMenu, bindTrigger } from 'material-ui-popup-state';

import openGINATriggerFileDialog from '../../dialogs/importGINAFile';
import {
  $activeTriggerTag,
  $topLevel,
  activateTriggerTagID,
  applyDeltas,
} from '../../features/triggers/triggersSlice';
import { TriggerGroupDescendant } from '../../generated/TriggerGroupDescendant';
import { createTriggerTag } from '../../ipc';
import StandardTooltip from '../../widgets/StandardTooltip';
import TriggerGroupListItem from './TriggerGroupListItem';
import TriggerIDsInSelectedTriggerTagContext from './TriggerIDsInSelectedTriggerTagContext';
import TriggerListItem from './TriggerListItem';
import TriggerTagChanger from './TriggerTagChanger';

import './TriggerTree.css';

const TriggerTree: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const triggerTreeMoreMenuID = useId();
  const top: TriggerGroupDescendant[] = useSelector($topLevel);
  const activeTriggerTag = useSelector($activeTriggerTag);

  const activeTriggersSet = activeTriggerTag
    ? {
        tagID: activeTriggerTag.id,
        triggerIDs: new Set(activeTriggerTag.triggers),
      }
    : null;

  return (
    <div className="trigger-tree trigger-browser-scrollable-container">
      <div className="trigger-browser-scrollable-content scrollable-content central-content">
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
              }
            }}
          />

          <PopupState variant="popover" popupId={triggerTreeMoreMenuID}>
            {(popupState) => (
              <>
                <div>
                  <StandardTooltip help="Search" placement="left">
                    <span>
                      {/* span is needed because button is disabled and disabled buttons don't fire any events */}
                      <IconButton sx={{ color: 'black' }} disabled={true}>
                        <ManageSearch />
                      </IconButton>
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
            {top.length ? (
              <ul>
                {top.map((tgd) =>
                  tgd.variant === 'T' ? (
                    <TriggerListItem key={tgd.value} triggerID={tgd.value} />
                  ) : (
                    <TriggerGroupListItem key={tgd.value} groupID={tgd.value} />
                  )
                )}
              </ul>
            ) : (
              <p>You have not created any triggers yet.</p>
            )}
          </div>
        </TriggerIDsInSelectedTriggerTagContext.Provider>
      </div>
    </div>
  );
};

export default TriggerTree;
