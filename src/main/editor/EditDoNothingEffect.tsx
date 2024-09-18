import EffectWithoutOptions from './EffectWithoutOptions';

const EditDoNothingEffect: React.FC<{ onDelete: () => void }> = ({
  onDelete,
}) => (
  <EffectWithoutOptions
    variant="DoNothing"
    help="Does nothing and finishes immediately. This can be useful as a placeholder effect."
    onDelete={onDelete}
  />
);

export default EditDoNothingEffect;
