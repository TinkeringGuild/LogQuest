import { useSelector } from 'react-redux';

import { AudioFileOutlined, PlayCircleOutline } from '@mui/icons-material';
import Button from '@mui/material/Button';

import {
  EffectVariantPlayAudioFile,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import EffectWithOptions from './EffectWithOptions';

const EditPlayAudioFileEffect: React.FC<{
  selector: TriggerEditorSelector<EffectVariantPlayAudioFile>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const { value: filePath } = useSelector(triggerEditorSelector(selector));
  return (
    <EffectWithOptions
      variant="PlayAudioFile"
      help="Plays a sound file located in your LogQuest configuration directory"
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
