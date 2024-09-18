import { QuestionMarkOutlined } from '@mui/icons-material';
import { useSelector } from 'react-redux';

import {
  EffectVariantCopyToClipboard,
  EffectVariantOverlayMessage,
  EffectVariantPause,
  EffectVariantPlayAudioFile,
  EffectVariantRunSystemCommand,
  EffectVariantSpeak,
  triggerEditorSelector,
  TriggerEditorSelector,
  TriggerEditorState,
} from '../../features/triggers/triggerEditorSlice';
import { Effect } from '../../generated/Effect';
import { EffectWithID } from '../../generated/EffectWithID';
import { UUID } from '../../generated/UUID';
import EditCopyToClipboardEffect from './EditCopyToClipboardEffect';
import EditOverlayMessageEffect from './EditOverlayMessageEffect';
import EditPauseEffect from './EditPauseEffect';
import EditPlayAudioFileEffect from './EditPlayAudioFileEffect';
import EditRunSystemCommandEffect from './EditRunSystemCommandEffect';
import EditScopedTimerEffect from './EditScopedTimerEffect';
import EditSequenceEffect from './EditSequenceEffect';
import EditSpeakEffect from './EditSpeakEffect';
import EditSpeakStopEffect from './EditSpeakStopEffect';
import EditStartTimerEffect from './EditStartTimerEffect';
import EffectWithOptions from './EffectWithOptions';

type EffectVariantScopedTimer = Extract<
  Effect,
  { variant: 'ScopedTimerEffect' }
>;

type EffectVariantSequence = Extract<Effect, { variant: 'Sequence' }>;

type EffectVariantStartTimer = Extract<Effect, { variant: 'StartTimer' }>;

// Functions that begin with '$$' indicate a selector that operates on slice-state
// rather than store-wide state.
function $$innerAs<T extends Effect>(
  effectSelector: TriggerEditorSelector<EffectWithID>
): (slice: TriggerEditorState) => T {
  return (slice) => {
    const effect: EffectWithID = effectSelector(slice);
    return effect.inner as T;
  };
}

const EditEffect: React.FC<{
  triggerID: UUID;
  effectSelector: TriggerEditorSelector<EffectWithID>;
  onDelete: () => void;
}> = ({ triggerID, effectSelector, onDelete }) => {
  const effect = useSelector(triggerEditorSelector(effectSelector));

  switch (effect.inner.variant) {
    case 'CopyToClipboard':
      return (
        <EditCopyToClipboardEffect
          selector={$$innerAs<EffectVariantCopyToClipboard>(effectSelector)}
          onDelete={onDelete}
        />
      );
    case 'Speak':
      return (
        <EditSpeakEffect
          selector={$$innerAs<EffectVariantSpeak>(effectSelector)}
          onDelete={onDelete}
        />
      );
    case 'Sequence':
      return (
        <EditSequenceEffect
          triggerID={triggerID}
          seqSelector={(slice: TriggerEditorState) => {
            const sequenceEffect =
              $$innerAs<EffectVariantSequence>(effectSelector)(slice);
            return sequenceEffect.value;
          }}
          onDelete={onDelete}
        />
      );
    case 'StartTimer':
      return (
        <EditStartTimerEffect
          timerSelector={(slice: TriggerEditorState) => {
            const startTimerEffect =
              $$innerAs<EffectVariantStartTimer>(effectSelector)(slice);
            return startTimerEffect.value;
          }}
          onDelete={onDelete}
        />
      );
    case 'ScopedTimerEffect':
      return (
        <EditScopedTimerEffect
          triggerID={triggerID}
          timerSelector={(slice: TriggerEditorState) => {
            const scopedTimerEffect =
              $$innerAs<EffectVariantScopedTimer>(effectSelector)(slice);
            return scopedTimerEffect.value;
          }}
          onDelete={onDelete}
        />
      );
    case 'OverlayMessage':
      return (
        <EditOverlayMessageEffect
          selector={$$innerAs<EffectVariantOverlayMessage>(effectSelector)}
          onDelete={onDelete}
        />
      );

    case 'PlayAudioFile':
      return (
        <EditPlayAudioFileEffect
          selector={$$innerAs<EffectVariantPlayAudioFile>(effectSelector)}
          onDelete={onDelete}
        />
      );
    case 'Pause':
      return (
        <EditPauseEffect
          selector={$$innerAs<EffectVariantPause>(effectSelector)}
          onDelete={onDelete}
        />
      );
    case 'RunSystemCommand':
      return (
        <EditRunSystemCommandEffect
          selector={$$innerAs<EffectVariantRunSystemCommand>(effectSelector)}
          onDelete={onDelete}
        />
      );
    case 'SpeakStop':
      return <EditSpeakStopEffect onDelete={onDelete} />;
    case 'Parallel':
    case 'DoNothing':
    case 'StartStopwatch':
    default:
      return (
        <EffectWithOptions
          title={effect.inner.variant}
          help="TODO"
          icon={<QuestionMarkOutlined />}
          onDelete={onDelete}
        >
          <h3>TODO!</h3>
          <pre>{JSON.stringify(effect.inner, null, 2)}</pre>
        </EffectWithOptions>
      );
  }
};

export default EditEffect;
