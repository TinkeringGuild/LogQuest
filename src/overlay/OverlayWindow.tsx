import "../base.css";
import "./OverlayWindow.css";
import Countdown from "./Countdown";
import DynamicContainer from "./DynamicContainer";
import { listen } from "@tauri-apps/api/event";
import { useState, useEffect } from "react";

function OverlayWindow() {
  const [editable, setEditable] = useState(false);

  useEffect(() => {
    const unlisten = listen("editable-changed", (event) => {
      setEditable(!!event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  return (
    <div className={`overlay ${editable ? "is-editable" : "is-static"}`}>
      <DynamicContainer width={450} height={500}>
        <Countdown label="Clarity" duration={5} />
        <Countdown label="Visions of Grandeur" duration={60} />
      </DynamicContainer>
    </div>
  );
}

export default OverlayWindow;
