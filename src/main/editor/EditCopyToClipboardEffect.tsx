import { ContentPasteOutlined } from '@mui/icons-material';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import TextField from '@mui/material/TextField';
import { useSelector, useDispatch } from 'react-redux';

import {
  triggerEditorSelector,
  TriggerEditorSelector,
  EffectVariantCopyToClipboard,
  setCopyToClipboardTemplate,
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
              title="Copy to Clipboard"
              help="Copies the text to the system clipboard"
              icon={<ContentPasteOutlined />}
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
