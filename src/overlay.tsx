import React from 'react';
import ReactDOM from 'react-dom/client';
import { Provider } from 'react-redux';

import OverlayWindow from './overlay/OverlayWindow';
import store from './OverlayStore';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <Provider store={store}>
    <React.StrictMode>
      <OverlayWindow />
    </React.StrictMode>
  </Provider>
);

console.log('STARTING OVERLAY');
