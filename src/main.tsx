import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import store from './MainStore';
import MainWindow from './main/MainWindow';

import '@fontsource/roboto/latin-300.css';
import '@fontsource/roboto/latin-400.css';
import '@fontsource/roboto/latin-500.css';
import '@fontsource/roboto/latin-700.css';
import '@fontsource/roboto-mono/latin-300.css';
import '@fontsource/roboto-mono/latin-400.css';
import '@fontsource/roboto-mono/latin-500.css';
import '@fontsource/roboto-mono/latin-700.css';

import './base.css';

const container = document.getElementById('root') as HTMLElement;
const root = createRoot(container);

// React.StrictMode isn't used because it doubly renders the top-level component.
root.render(
  <Provider store={store}>
    <MainWindow />
  </Provider>
);
