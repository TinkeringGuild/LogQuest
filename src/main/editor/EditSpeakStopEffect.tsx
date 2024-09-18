import EffectWithoutOptions from './EffectWithoutOptions';

const EditSpeakStopEffect: React.FC<{ onDelete: () => void }> = ({
  onDelete,
}) => (
  <EffectWithoutOptions
    variant="SpeakStop"
    help="Immediately interrupts any playing text-to-speech"
    onDelete={onDelete}
  />
);

export default EditSpeakStopEffect;
