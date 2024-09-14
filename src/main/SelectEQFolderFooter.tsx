import React, { CSSProperties } from 'react';
import { useDispatch } from 'react-redux';
import Button from '@mui/material/Button';
import CreateNewFolderOutlinedIcon from '@mui/icons-material/CreateNewFolderOutlined';

import openEQFolderSelectionDialog from '../dialogs/selectEQDir';

const SelectEQFolderFooter: React.FC<{}> = () => {
  const dispatch = useDispatch();
  return (
    <footer style={styleFooter}>
      <div style={{ display: 'inline-block' }}>
        <Button
          size="large"
          variant="contained"
          startIcon={<CreateNewFolderOutlinedIcon />}
          className="footer-cta"
          onClick={() => openEQFolderSelectionDialog(dispatch)}
          sx={{
            color: 'black',
            backgroundColor: 'yellow',
            '&:hover': { backgroundColor: 'white' },
          }}
        >
          Select EverQuest Folder
        </Button>
      </div>
      <p style={{ margin: '3px 0 0', padding: 0 }}>
        To use LogQuest, select the folder where EverQuest is installed.
      </p>
    </footer>
  );
};

const styleFooter: CSSProperties = {
  flexShrink: 0,
  textAlign: 'center',
  backgroundColor: 'black',
  color: 'white',
  padding: '10px 0',
};

export default SelectEQFolderFooter;
