import { ContentPasteOutlined } from '@mui/icons-material';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import TextField from '@mui/material/TextField';
import { useSelector, useDispatch } from 'react-redux';

import {
  editorSelector,
  EditorSelector,
  EffectVariantCopyToClipboard,
  setCopyToClipboardTemplate,
} from '../../features/triggers/editorSlice';
import { EffectHeader, EffectTitle } from './widgets/EffectHeader';

const EditCopyToClipboardEffect: React.FC<{
  selector: EditorSelector<EffectVariantCopyToClipboard>;
  onDelete: () => void;
}> = ({ selector, onDelete }) => {
  const dispatch = useDispatch();
  const { value: tmpl } = useSelector(editorSelector(selector));
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
