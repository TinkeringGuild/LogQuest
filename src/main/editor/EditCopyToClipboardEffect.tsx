import { useDispatch, useSelector } from 'react-redux';

import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import TextField from '@mui/material/TextField';

import {
  EffectVariantCopyToClipboard,
  setCopyToClipboardTemplate,
  triggerEditorSelector,
  TriggerEditorSelector,
} from '../../features/triggers/triggerEditorSlice';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';

const EditCopyToClipboardEffect: React.FC<{
  selector: TriggerEditorSelector<EffectVariantCopyToClipboard>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const dispatch = useDispatch();
  const { value: tmpl } = useSelector(triggerEditorSelector(selector));
  return (
    <Card elevation={10}>
      <CardHeader
        title={
          <EffectHeader onDelete={onDelete}>
            <EffectTitle
              variant="CopyToClipboard"
              help="Copies the text to the system clipboard"
            />
          </EffectHeader>
        }
      />
      <CardContent>
        <TextField
          label="Clipboard Text (Template)"
          defaultValue={tmpl}
          fullWidth
          onBlur={(e) =>
            dispatch(
              setCopyToClipboardTemplate({ tmpl: e.target.value, selector })
            )
          }
        />
      </CardContent>
    </Card>
  );
};

export default EditCopyToClipboardEffect;
