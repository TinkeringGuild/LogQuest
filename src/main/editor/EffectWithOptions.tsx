import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import { ReactElement, ReactNode } from 'react';

import { EffectHeader, EffectTitle } from './widgets/EffectHeader';

const EffectWithOptions: React.FC<{
  title: string;
  help: string;
  icon: ReactElement;
  children: ReactNode;
  onDelete: () => void;
}> = ({ title, help, icon, children, onDelete }) => (
  <Card elevation={10}>
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
