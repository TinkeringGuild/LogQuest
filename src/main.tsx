import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import store from './MainStore';

import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';
import './base.css';
import MainWindow from './main/MainWindow';

const container = document.getElementById('root') as HTMLElement;
const root = createRoot(container);

// React.StrictMode isn't used because it doubly renders the top-level component.
root.render(
  <Provider store={store}>
    <MainWindow />
  </Provider>
);
