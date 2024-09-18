import Card from '@mui/material/Card';
import CardHeader from '@mui/material/CardHeader';

import { EffectHeader, EffectTitle } from './widgets/EffectHeader';
import { EffectVariant } from './effect-utils';

const EffectWithoutOptions: React.FC<{
  variant: EffectVariant;
  help: string;
  onDelete: () => void;
}> = ({ variant, help, onDelete }) => (
  <Card elevation={10} sx={{ width: 300, alignSelf: 'center' }}>
    <CardHeader
      sx={{ textAlign: 'center', justifyContent: 'center' }}
      title={
        <EffectHeader onDelete={onDelete}>
          <EffectTitle variant={variant} help={help} />
        </EffectHeader>
      }
    />
  </Card>
);

export default EffectWithoutOptions;
