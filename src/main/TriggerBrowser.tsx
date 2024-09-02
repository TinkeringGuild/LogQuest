import React from 'react';
import { useSelector } from 'react-redux';

import TriggerTree from './TriggerTree';
import { $currentTrigger } from '../features/triggers/triggersSlice';
import TriggerEditor from './TriggerEditor';

import './TriggerBrowser.css';

const TriggerBrowser: React.FC<{}> = () => {
  const currentTrigger = useSelector($currentTrigger);
  return (
    <div className="trigger-browser">
      <TriggerTree />
      {currentTrigger && <TriggerEditor trigger={currentTrigger} />}
    </div>
  );
};

export default TriggerBrowser;
