import { SxProps } from '@mui/material';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import React, { CSSProperties, ReactNode } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import {
  $currentMode,
  $isBootstrapped,
  $isLoading,
  MODE,
  navigateTo,
} from '../features/app/appSlice';
import { $eqDirBlank } from '../features/config/configSlice';
import SelectEQFolderFooter from './SelectEQFolderFooter';

const Layout: React.FC<{ children: ReactNode }> = ({ children }) => {
  const bootstrapped = useSelector($isBootstrapped);
  const needsEQDir = useSelector($eqDirBlank);

  return (
    <div id="layout" style={styleLayout}>
      <div id="central-row" style={styleCentral}>
        <NavSidebar />
        {children}
      </div>
      {bootstrapped && needsEQDir && <SelectEQFolderFooter />}
    </div>
  );
};

const NavSidebar: React.FC<{}> = () => {
  const mode = useSelector($currentMode);
  return (
    <div style={styleNavSidebar}>
      <Stack>
        <NavButton to="overview" current={mode} size="large" text="Overview" />
        <NavButton to="triggers" current={mode} size="large" text="Triggers" />
        <NavButton to="overlay" current={mode} size="large" text="Overlay" />
      </Stack>
      <Stack>
        <NavButton to="settings" current={mode} size="medium" text="Settings" />
        <NavButton to="about" current={mode} size="medium" text="About" />
        <NavButton to="help" current={mode} size="medium" text="Help" />
      </Stack>
    </div>
  );
};

const NavButton: React.FC<{
  text: string;
  to: MODE;
  current: MODE;
  size: 'small' | 'medium' | 'large';
}> = ({ to, current, text, size }) => {
  const dispatch = useDispatch();
  const isLoading = useSelector($isLoading);
  const active = to == current;

  const inactiveStyle: SxProps = {
    padding: '10px 25px',
    color: 'black',
    borderRadius: 0,
    '&:hover': {
      backgroundColor: 'white',
    },
    '&:active': {
      backgroundColor: 'black',
      color: 'white',
    },
  };
  const activeStyle: SxProps = {
    ...inactiveStyle,
    color: 'white',
    backgroundColor: 'black',
    '&:hover': {
      cursor: 'default',
      backgroundColor: 'black',
    },
    '&:active': {
      backgroundColor: 'black',
    },
    '&:disabled': {
      color: '#777',
    },
  };
  return (
    <Button
      size={size}
      className={active ? 'nav-active' : ''}
      sx={active ? activeStyle : inactiveStyle}
      disabled={isLoading}
      onClick={() => dispatch(navigateTo(to))}
    >
      {text}
    </Button>
  );
};

const styleLayout: CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  height: '100vh',
};

const styleCentral: CSSProperties = {
  display: 'flex',
  flexGrow: 1,
  flexDirection: 'row',
};

const styleNavSidebar: CSSProperties = {
  display: 'flex',
  flex: 0,
  alignItems: 'stretch',
  flexDirection: 'column',
  justifyContent: 'space-between',
  backgroundColor: '#eee',
};

export default Layout;
