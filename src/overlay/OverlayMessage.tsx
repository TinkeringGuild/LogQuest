import React from 'react';

import './OverlayMessage.css';

interface OverlayMessageProps {
  text: string;
}

const OverlayMessage: React.FC<OverlayMessageProps> = ({ text }) => (
  <p className="overlay-message text-outline">
    <span>{text}</span>
  </p>
);

export default OverlayMessage;
