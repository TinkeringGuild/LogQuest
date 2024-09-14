import React from 'react';
import { useDispatch, useSelector } from 'react-redux';

import DownloadingIcon from '@mui/icons-material/Downloading';
import { Box } from '@mui/material';
import Button from '@mui/material/Button';

import openGINATriggerFileDialog from '../../dialogs/importGINAFile';
import {
  $activeTriggerTag,
  $topLevel,
  activateTriggerTagID,
  applyDeltas,
} from '../../features/triggers/triggersSlice';
import { TriggerGroupDescendant } from '../../generated/TriggerGroupDescendant';
import { createTriggerTag } from '../../ipc';
import TriggerGroupListItem from './TriggerGroupListItem';
import TriggerIDsInSelectedTriggerTagContext from './TriggerIDsInSelectedTriggerTagContext';
import TriggerListItem from './TriggerListItem';
import TriggerTagChanger from './TriggerTagChanger';

import './TriggerTree.css';

const TriggerTree: React.FC<{}> = () => {
  const dispatch = useDispatch();
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
        <Box justifyItems="right">
          <div style={{ textAlign: 'right' }}>
            <Button
              size="small"
              variant="contained"
              startIcon={<DownloadingIcon />}
              onClick={() => openGINATriggerFileDialog(dispatch)}
            >
              Import GINA Export
            </Button>
          </div>
        </Box>

        <TriggerTagChanger
          onChange={(tagIDMaybe) => dispatch(activateTriggerTagID(tagIDMaybe))}
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
