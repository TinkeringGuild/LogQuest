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
      <div id="central" style={styleCentral}>
        <NavSidebar />
        <div id="main-flex-container" style={styleMainFlexContainer}>
          {children}
        </div>
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
  const isLoading = useSelector($isLoading);
  const dispatch = useDispatch();
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
  flex: 1,
  flexDirection: 'column',
  height: '100vh',
};

const styleHeader: CSSProperties = {
  flex: 0,
  backgroundColor: 'black',
  color: 'white',
  fontFamily: 'Jacquard_12, Roboto, "Helvetica Neue", Helvetica',
  letterSpacing: -2,
  fontSize: '25px',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: '10px 0',
};

const styleCentral: CSSProperties = {
  display: 'flex',
  flex: 1,
  flexDirection: 'row',
};

const styleNavSidebar: CSSProperties = {
  display: 'flex',
  flex: 0,
  alignItems: 'stretch',
  flexDirection: 'column',
  justifyContent: 'space-between',
  backgroundColor: '#eee',
  // padding: '15px',
  marginRight: '15px',
};

const styleMainFlexContainer: CSSProperties = {
  flex: 1,
  position: 'relative',
};

export default Layout;
