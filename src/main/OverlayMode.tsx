import Grid from '@mui/material/Grid';
import Slider from '@mui/material/Slider';
import React from 'react';
import { useDispatch, useSelector } from 'react-redux';

import {
  $overlayOpacity,
  setOverlayOpacity,
} from '../features/overlay/overlaySlice';
import { DEFAULT_OVERLAY_OPACITY } from '../generated/constants';

const DEFAULT_OVERLAY_TRANSPARENCY = 100 - DEFAULT_OVERLAY_OPACITY;

const OverlayMode: React.FC<{}> = () => {
  const opacity = useSelector($overlayOpacity);
  const transparency = 100 - opacity;

  const dispatch = useDispatch();
  return (
    <div className="overlay-mode central-content">
      <h1>Overlay</h1>

      <div>
        <Grid container spacing={2}>
          <Grid item xs={3}>
            <strong>Transparency</strong>{' '}
          </Grid>
          <Grid item sx={{ width: 200 }}>
            <Slider
              sx={{ marginTop: '-3px' }}
              value={transparency}
              valueLabelDisplay="on"
              marks={[{ value: DEFAULT_OVERLAY_TRANSPARENCY }]}
              valueLabelFormat={(value) =>
                value === DEFAULT_OVERLAY_TRANSPARENCY
                  ? `${value}% (Default)`
                  : `${value}%`
              }
              onChange={(_, value) => {
                dispatch(setOverlayOpacity(100 - (value as number)));
              }}
            />
          </Grid>
        </Grid>
      </div>

      <p>
        <strong>Toggle Overlay Edit Mode shortcut:</strong>{' '}
        <kbd style={{ fontSize: 15 }}>Ctrl+Alt+Shift+L</kbd>
      </p>
    </div>
  );
};

export default OverlayMode;
