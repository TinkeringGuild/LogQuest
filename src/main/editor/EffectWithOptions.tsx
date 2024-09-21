import { ReactNode } from 'react';

import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';

import { EffectVariant, TimerEffectVariant } from './effect-utils';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';

const EffectWithOptions: React.FC<{
  variant: EffectVariant | TimerEffectVariant;
  help: string;
  width?: number;
  children: ReactNode;
  onDelete: () => void;
}> = ({ variant, help, width, children, onDelete }) => (
  <Card
    className="effect-with-options"
    elevation={10}
    sx={{ ...(width ? { width, alignSelf: 'center' } : {}) }}
  >
    <CardHeader
      title={
        <EffectHeader onDelete={onDelete}>
          <EffectTitle variant={variant} help={help} />
        </EffectHeader>
      }
    />
    <CardContent>{children}</CardContent>
  </Card>
);

export default EffectWithOptions;
