import { useDispatch, useSelector } from 'react-redux';

import { AudioFileOutlined, PlayCircleOutline } from '@mui/icons-material';
import Button from '@mui/material/Button';
import RemoveCircleOutline from '@mui/icons-material/RemoveCircleOutline';

import {
  EffectVariantPlayAudioFile,
  setAudioFile,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import EffectWithOptions from './EffectWithOptions';
import selectAudioFileDialog from '../../dialogs/selectAudioFile';
import { playAudioFile } from '../../ipc';

const EditPlayAudioFileEffect: React.FC<{
  selector: TriggerEditorSelector<EffectVariantPlayAudioFile>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const dispatch = useDispatch();
  const { value: filePath } = useSelector(triggerEditorSelector(selector));

  return (
    <EffectWithOptions
      variant="PlayAudioFile"
      help="Plays a sound file located in your LogQuest configuration directory"
      onDelete={onDelete}
    >
      <p style={{ marginTop: 0 }}>
        {filePath ? <code>{filePath}</code> : 'No audio file selected'}
      </p>
      <Button
        variant="contained"
        startIcon={<AudioFileOutlined />}
        onClick={async () => {
          const path = await selectAudioFileDialog();
          if (path) {
            dispatch(setAudioFile({ path, selector }));
          }
        }}
      >
        Select File
      </Button>{' '}
      {filePath && (
        <>
          <Button
            variant="outlined"
            startIcon={<PlayCircleOutline />}
            onClick={() => {
              playAudioFile(filePath);
            }}
          >
            Test Playback
          </Button>{' '}
          <Button
            variant="outlined"
            startIcon={<RemoveCircleOutline />}
            onClick={() => {
              dispatch(setAudioFile({ path: null, selector }));
            }}
          >
            Deselect File
          </Button>
        </>
      )}
    </EffectWithOptions>
  );
};

export default EditPlayAudioFileEffect;
