import React from 'react';
import { useSelector } from 'react-redux';

import { $draftTrigger } from '../features/triggers/triggerEditorSlice';
import TriggerTree from './TriggerTree';
import TriggerEditor from './editor/TriggerEditor';

import './TriggerBrowser.css';

const TriggerBrowser: React.FC<{}> = () => {
  const currentTrigger = useSelector($draftTrigger);
  return (
    <div className="trigger-browser">
      {currentTrigger ? <TriggerEditor /> : <TriggerTree />}
    </div>
  );
};

export default TriggerBrowser;
