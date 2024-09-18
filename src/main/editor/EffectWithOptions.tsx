import { ReactElement, ReactNode } from 'react';

import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';

import { EffectHeader, EffectTitle } from './widgets/EffectHeader';

const EffectWithOptions: React.FC<{
  title: string;
  help: string;
  icon: ReactElement;
  width?: number;
  children: ReactNode;
  onDelete: () => void;
}> = ({ title, help, icon, width, children, onDelete }) => (
  <Card
    elevation={10}
    sx={{ ...(width ? { width, alignSelf: 'center' } : {}) }}
  >
    <CardHeader
      title={
        <EffectHeader onDelete={onDelete}>
          <EffectTitle title={title} help={help} icon={icon} />
        </EffectHeader>
      }
    />
    <CardContent>{children}</CardContent>
  </Card>
);

export default EffectWithOptions;
