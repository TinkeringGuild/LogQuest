// import React from "react";
import { createRoot } from "react-dom/client";
import { Provider } from "react-redux";
import store from "./store";
import MainWindow from "./main/MainWindow";

const container = document.getElementById("root") as HTMLElement;
const root = createRoot(container);

root.render(
  <Provider store={store}>
    <MainWindow />,
  </Provider>,
);

//// StrictMode is disabled because it doubly renders the top-level component.
// <React.StrictMode>
//   <MainWindow />
// </React.StrictMode>,
