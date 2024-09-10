import {
  AudioFileOutlined,
  HourglassTopOutlined,
  InsertCommentOutlined,
  PlayCircleOutline,
  VolumeUpOutlined,
} from '@mui/icons-material';
import Button from '@mui/material/Button';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import TextField from '@mui/material/TextField';
import { useSelector } from 'react-redux';

import {
  editorSelector,
  EditorSelector,
  EditorState,
  EffectVariantCopyToClipboard,
  EffectVariantSpeak,
} from '../../features/triggers/editorSlice';
import { Effect } from '../../generated/Effect';
import { EffectWithID } from '../../generated/EffectWithID';
import { UUID } from '../../generated/UUID';
import EditCopyToClipboardEffect from './EditCopyToClipboardEffect';
import EditScopedTimerEffect from './EditScopedTimerEffect';
import EditSequenceEffect from './EditSequenceEffect';
import EditSpeakEffect from './EditSpeakEffect';
import EditStartTimerEffect from './EditStartTimerEffect';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';

type EffectVariantScopedTimer = Extract<
  Effect,
  { variant: 'ScopedTimerEffect' }
>;

type EffectVariantSequence = Extract<Effect, { variant: 'Sequence' }>;

type EffectVariantStartTimer = Extract<Effect, { variant: 'StartTimer' }>;

const EditEffect: React.FC<{
  triggerID: UUID;
  effectSelector: EditorSelector<EffectWithID>;
  onDelete: () => void;
}> = ({ triggerID, effectSelector, onDelete }) => {
  const effect = useSelector(editorSelector(effectSelector));

  switch (effect.inner.variant) {
    case 'CopyToClipboard':
      return (
        <EditCopyToClipboardEffect
          selector={(slice) => {
            const effect: EffectWithID = effectSelector(slice);
            return effect.inner as EffectVariantCopyToClipboard;
          }}
          onDelete={onDelete}
        />
      );
    case 'Speak':
      return (
        <EditSpeakEffect
          selector={(slice) => {
            const effect: EffectWithID = effectSelector(slice);
            return effect.inner as EffectVariantSpeak;
          }}
          onDelete={onDelete}
        />
      );
    case 'Sequence':
      return (
        <EditSequenceEffect
          triggerID={triggerID}
          seqSelector={(slice: EditorState) => {
            const effect: EffectWithID = effectSelector(slice);
            const scopedTimerEffect = effect.inner as EffectVariantSequence;
            return scopedTimerEffect.value;
          }}
          onDelete={onDelete}
        />
      );
    case 'StartTimer':
      return (
        <EditStartTimerEffect
          timerSelector={(slice: EditorState) => {
            const effect: EffectWithID = effectSelector(slice);
            const scopedTimerEffect = effect.inner as EffectVariantStartTimer;
            return scopedTimerEffect.value;
          }}
          onDelete={onDelete}
        />
      );
    case 'ScopedTimerEffect':
      return (
        <EditScopedTimerEffect
          triggerID={triggerID}
          timerSelector={(slice: EditorState) => {
            const effect: EffectWithID = effectSelector(slice);
            const scopedTimerEffect = effect.inner as EffectVariantScopedTimer;
            return scopedTimerEffect.value;
          }}
          onDelete={onDelete}
        />
      );
    case 'OverlayMessage':
      return (
        <Card elevation={10}>
          <CardHeader
            title={
              <EffectHeader onDelete={onDelete}>
                <EffectTitle
                  title="Show Overlay Message"
                  help="Shows a message on the Overlay."
                  icon={<InsertCommentOutlined />}
                />
              </EffectHeader>
            }
          />
          <CardContent>
            <TextField
              label="Overlay Message (Template)"
              fullWidth
              value={effect.inner.value}
            />
          </CardContent>
        </Card>
      );

    case 'PlayAudioFile':
      return (
        <Card elevation={10}>
          <CardHeader
            title={
              <EffectHeader onDelete={onDelete}>
                <EffectTitle
                  title="Play Audio File"
                  help="Plays an sound file located in your LogQuest configuration directory"
                  icon={<VolumeUpOutlined />}
                />
              </EffectHeader>
            }
          />
          <CardContent>
            <p>
              {effect.inner.value ? (
                <code>{effect.inner.value}</code>
              ) : (
                'No audio file selected'
              )}
            </p>
            <Button variant="contained" startIcon={<AudioFileOutlined />}>
              Select file
            </Button>{' '}
            {!effect.inner.value && (
              <Button variant="outlined" startIcon={<PlayCircleOutline />}>
                Test playback
              </Button>
            )}
          </CardContent>
        </Card>
      );
    case 'Pause':
    case 'Parallel':
    case 'DoNothing':
    case 'RunSystemCommand':
    case 'SpeakStop':
    case 'StartStopwatch':
    default:
      return (
        <Card elevation={10}>
          <CardHeader
            title={
              <EffectHeader onDelete={onDelete}>
                <EffectTitle
                  title={effect.inner.variant}
                  help="TODO"
                  icon={<HourglassTopOutlined />}
                />
              </EffectHeader>
            }
          ></CardHeader>
          <CardContent>{effect.inner.variant}</CardContent>
        </Card>
      );
  }
};

export default EditEffect;
