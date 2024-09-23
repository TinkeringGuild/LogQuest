import { sortBy } from 'lodash';
import React, { useEffect, useMemo } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { FormControlLabel, Stack, Switch } from '@mui/material';

import {
  $activeTriggerTags,
  $currentCharacter,
  updateActivedTriggerTagIDs,
} from '../features/app/appSlice';
import { $triggerTags } from '../features/triggers/triggersSlice';
import { UUID } from '../generated/UUID';
import { getActiveTriggerTags, setTriggerTagActivated } from '../ipc';
import { LQ_VERSION } from '../generated/constants';

const OverviewMode: React.FC<{}> = () => {
  const dispatch = useDispatch();
  const currentCharacter = useSelector($currentCharacter);
  const triggerTags = useSelector($triggerTags);
  const activeTriggerTagIDs = useSelector($activeTriggerTags);

  const activeTriggerTagIDsSet: Set<UUID> = useMemo(
    () => new Set(activeTriggerTagIDs || []),
    [activeTriggerTagIDs]
  );

  const sortedTags = useMemo(
    () => sortBy(Object.values(triggerTags), (tag) => tag.name.toUpperCase()),
    [triggerTags]
  );

  useEffect(() => {
    let isMounted = true;
    getActiveTriggerTags().then((triggerTags) => {
      if (isMounted) {
        dispatch(updateActivedTriggerTagIDs(triggerTags));
      }
    });
    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <div className="overview-mode central-content">
      <div style={{ textAlign: 'center' }}>
        <img
          width="202"
          height="44"
          src="/LogQuest header black.png"
          alt="LogQuest"
        />
        <p style={{ marginTop: 0, fontSize: '17px', fontFamily: 'monospace' }}>
          version {LQ_VERSION.join('.')}
        </p>
      </div>
      <h3>
        {currentCharacter
          ? `Current character: ${currentCharacter.name}`
          : 'No current character detected'}
      </h3>
      <h2 style={{ marginBottom: 5 }}>Activate Trigger Tags</h2>
      <Stack gap={0}>
        {sortedTags.map((tag) => {
          return (
            <FormControlLabel
              key={tag.id}
              checked={activeTriggerTagIDsSet.has(tag.id)}
              label={tag.name}
              control={<Switch />}
              onChange={async (_, checked) => {
                const activatedIDs = await setTriggerTagActivated(
                  tag.id,
                  checked
                );
                dispatch(updateActivedTriggerTagIDs(activatedIDs));
              }}
            />
          );
        })}
      </Stack>
    </div>
  );
};

export default OverviewMode;
