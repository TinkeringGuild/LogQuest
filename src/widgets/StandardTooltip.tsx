import { ReactElement } from 'react';

import Tooltip, { TooltipProps } from '@mui/material/Tooltip';

const StandardTooltip: React.FC<{
  help: string;
  placement?: TooltipProps['placement'];
  children: ReactElement;
}> = ({ help, children, placement = 'top' }) => (
  <Tooltip title={help} arrow followCursor placement={placement}>
    {children}
  </Tooltip>
);

export default StandardTooltip;
