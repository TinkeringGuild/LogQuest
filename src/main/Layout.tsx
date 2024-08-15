import React, { ReactNode, CSSProperties } from 'react';
import { useSelector } from 'react-redux';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';

import { selectEQDirIsBlank } from '../features/config/configSlice';
import SelectEQFolderFooter from './SelectEQFolderFooter';
import { hasBootstrapped } from '../features/app/appSlice';

const Layout: React.FC<{ children: ReactNode }> = ({ children }) => {
  const bootstrapped = useSelector(hasBootstrapped);
  const needsEQDir = useSelector(selectEQDirIsBlank);

  return (
    <div id="layout" style={styleLayout}>
      <header style={styleHeader}>
        <img
          width="202"
          height="44"
          src="/LogQuest header white.png"
          alt="LogQuest"
        />
      </header>
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
  const NavButton: React.FC<{
    text: string;
    size: 'small' | 'medium' | 'large';
  }> = ({ text, size }) => (
    <Button
      size={size}
      sx={{ padding: '10px 25px', color: 'black', borderRadius: 0 }}
    >
      {text}
    </Button>
  );
  return (
    <div style={styleNavSidebar}>
      <Stack>
        <NavButton size="large" text="Overview" />
        <NavButton size="large" text="Triggers" />
        <NavButton size="large" text="Overlay" />
      </Stack>
      <Stack>
        <NavButton size="medium" text="Settings" />
        <NavButton size="medium" text="Help" />
        <NavButton size="medium" text="About" />
      </Stack>
    </div>
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
