import { sortBy } from 'lodash';
import React, { useEffect, useMemo } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { FormControlLabel, Stack, Switch } from '@mui/material';

import './OverviewMode.css';

import {
  $activeTriggerTags,
  $currentCharacter,
  setCurrentCharacter,
  updateActivedTriggerTagIDs,
} from '../features/app/appSlice';
import { $triggerTags } from '../features/triggers/triggersSlice';
import { UUID } from '../generated/UUID';
import {
  getActiveTriggerTags,
  getCurrentCharacter,
  setTriggerTagActivated,
} from '../ipc';
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

  // Updates the current character and active trigger tags state
  useEffect(() => {
    let isMounted = true;

    getActiveTriggerTags().then((triggerTags) => {
      if (isMounted) {
        dispatch(updateActivedTriggerTagIDs(triggerTags));
      }
    });

    const updateCurrentCharacter = () => {
      if (!isMounted) return;
      getCurrentCharacter().then((characterMaybe) => {
        if (!isMounted) return;
        dispatch(setCurrentCharacter(characterMaybe));
      });
    };

    updateCurrentCharacter();

    const interval = setInterval(updateCurrentCharacter, 1000);

    return () => {
      isMounted = false;
      clearInterval(interval);
    };
  }, []);

  return (
    <div className="overview-mode central-content scrollable-container">
      <div className="scrollable-content">
        <div className="overview-mode-header">
          <img
            width="202"
            height="44"
            src="/LogQuest header black.png"
            alt="LogQuest"
          />
          <p>version {LQ_VERSION.join('.')}</p>
        </div>
        <div className="overview-mode-content">
          <div className="overview-mode-current-character">
            <h3>
              {currentCharacter
                ? `Current character: ${currentCharacter.name}`
                : 'No current character detected'}
            </h3>
            {!currentCharacter && (
              <p>
                When you begin playing a toon, LogQuest will automatically start
                watching its log file.
              </p>
            )}
          </div>
          <h2 style={{ marginBottom: 5 }}>Activated Trigger Tags</h2>
          {sortedTags.length ? (
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
          ) : (
            <>
              <p>You currently have no Trigger Tags.</p>
              <p>
                When you create one, you will be able to activate all Triggers
                associated with it. You can do this in the Triggers section.
              </p>
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default OverviewMode;
