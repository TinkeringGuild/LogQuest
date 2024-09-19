import { ReactElement } from 'react';

import { DeleteForeverOutlined } from '@mui/icons-material';
import Box from '@mui/material/Box';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import StandardTooltip from '../../../widgets/StandardTooltip';
import {
  EffectIcon,
  EffectVariant,
  HUMANIZED_EFFECT_NAMES,
} from '../effect-utils';

export const EffectTitle: React.FC<{
  variant: EffectVariant;
  help: string;
}> = ({ variant, help }) => {
  const VariantIcon = EffectIcon[variant];
  return (
    <ExplicitEffectTitle
      title={HUMANIZED_EFFECT_NAMES[variant]}
      help={help}
      icon={<VariantIcon />}
    />
  );
};

export const ExplicitEffectTitle: React.FC<{
  title: string;
  icon: ReactElement;
  help: string;
}> = ({ title, icon, help }) => {
  return (
    <Box display="flex" alignItems="center">
      <StandardTooltip help={help}>{icon}</StandardTooltip>
      <Typography variant="h6" sx={{ ml: 1 }}>
        {title}
      </Typography>
    </Box>
  );
};

export const EffectHeader: React.FC<{
  children: ReactElement;
  onDelete: () => void;
}> = ({ children, onDelete }) => {
  return (
    <Box
      className="effect-header"
      display="flex"
      flexDirection="row"
      alignItems="center"
    >
      <Box flexGrow={1} flexShrink={1} flexBasis={0}></Box>
      <Box flexGrow={0} flexShrink={0} flexBasis="auto">
        <Box display="flex" alignItems="center" justifyContent="center">
          {children}
        </Box>
      </Box>
      <Box
        flexGrow={1}
        flexShrink={1}
        flexBasis={0}
        display="flex"
        justifyContent="flex-end"
      >
        <StandardTooltip help="Delete this Effect">
          <Typography variant="h6" className="effect-delete-button">
            <IconButton onClick={onDelete}>
              <DeleteForeverOutlined />
            </IconButton>
          </Typography>
        </StandardTooltip>
      </Box>
    </Box>
  );
};
