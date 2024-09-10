import Tooltip from '@mui/material/Tooltip';
import { ReactElement } from 'react';

const StandardTooltip: React.FC<{ help: string; children: ReactElement }> = ({
  help,
  children,
}) => (
  <Tooltip title={help} arrow followCursor placement="top">
    {children}
  </Tooltip>
);

export default StandardTooltip;
