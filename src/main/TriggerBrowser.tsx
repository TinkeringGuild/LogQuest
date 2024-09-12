import React from 'react';
import { useSelector } from 'react-redux';

import { $triggerDraft } from '../features/triggers/triggerEditorSlice';
import TriggerTree from './TriggerTree';
import TriggerEditor from './editor/TriggerEditor';

import './TriggerBrowser.css';

const TriggerBrowser: React.FC<{}> = () => {
  const currentTrigger = useSelector($triggerDraft);
  return (
    <div className="trigger-browser">
      <TriggerTree />
      {currentTrigger && <TriggerEditor />}
    </div>
  );
};

export default TriggerBrowser;
