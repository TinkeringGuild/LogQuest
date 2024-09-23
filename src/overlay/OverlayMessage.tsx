import React from 'react';

import EmojiText from '../widgets/EmojiText';

import './OverlayMessage.css';

interface OverlayMessageProps {
  text: string;
}

const OverlayMessage: React.FC<OverlayMessageProps> = ({ text }) => (
  <p className="overlay-message text-outline">
    <span>
      <EmojiText text={text} />
    </span>
  </p>
);

export default OverlayMessage;
