import {
  AudioFileOutlined,
  PlayCircleOutline,
  VolumeUpOutlined,
} from '@mui/icons-material';
import { useSelector } from 'react-redux';

import {
  triggerEditorSelector,
  TriggerEditorSelector,
  EffectVariantPlayAudioFile,
} from '../../features/triggers/triggerEditorSlice';
import EffectWithOptions from './EffectWithOptions';
import Button from '@mui/material/Button';

const EditPlayAudioFileEffect: React.FC<{
  selector: TriggerEditorSelector<EffectVariantPlayAudioFile>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const { value: filePath } = useSelector(triggerEditorSelector(selector));
  return (
    <EffectWithOptions
      title="Play Audio File"
      help="Plays an sound file located in your LogQuest configuration directory"
      icon={<VolumeUpOutlined />}
      onDelete={onDelete}
    >
      <p>{filePath ? <code>{filePath}</code> : 'No audio file selected'}</p>
      <Button variant="contained" startIcon={<AudioFileOutlined />}>
        Select file
      </Button>{' '}
      {filePath && (
        <Button variant="outlined" startIcon={<PlayCircleOutline />}>
          Test playback
        </Button>
      )}
    </EffectWithOptions>
  );
};

export default EditPlayAudioFileEffect;
