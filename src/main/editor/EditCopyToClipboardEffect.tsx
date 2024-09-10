import { TemplateString } from '../../generated/TemplateString';
import { UUID } from '../../generated/UUID';

const EditCopyToClipboardEffect: React.FC<{
  triggerID: UUID;
  effectID: UUID;
  tmpl: TemplateString;
  onDelete: () => void;
}> = ({ tmpl, onDelete: _TODO }) => {
  return (
    <div>
      <p className="effect-name">Copy to Clipboard</p>
      <input type="text" value={tmpl} />
    </div>
  );
};

export default EditCopyToClipboardEffect;
