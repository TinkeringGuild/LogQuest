import Card from '@mui/material/Card';
import CardHeader from '@mui/material/CardHeader';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';
import VoiceOverOffOutlined from '@mui/icons-material/VoiceOverOffOutlined';

const EditSpeakStopEffect: React.FC<{ onDelete: () => void }> = ({
  onDelete,
}) => (
  <Card elevation={10} sx={{ width: 300, alignSelf: 'center' }}>
    <CardHeader
      sx={{ textAlign: 'center', justifyContent: 'center' }}
      title={
        <EffectHeader onDelete={onDelete}>
          <EffectTitle
            title="Stop Speaking"
            help="Immediately interrupts any playing text-to-speech"
            icon={<VoiceOverOffOutlined />}
          />
        </EffectHeader>
      }
    />
  </Card>
);

export default EditSpeakStopEffect;
