import "../base.css";
import "./MainWindow.css";
import { emit } from "@tauri-apps/api/event";
import { ReactNode } from "react";
import { SpellTimer } from "../types.tsx";
import { v4 as newUUID } from "uuid";

function MainWindow() {
  return (
    <div className="container">
      <p>Receive a buff!</p>

      <BuffButton name="Yaulp IV" duration={4 * 6} />
      <BuffButton name="Divine Aura" duration={3 * 6} />
      <BuffButton name="Clarity II" duration={35 * 60} />
      <BuffButton name="Visions of Grandeur" duration={42 * 60} />
      <BuffButton name="Focus of Spirit" duration={100 * 60} />
      <BuffButton name="Regrowth of the Grove" duration={19 * 60} />
      {/* <BuffButton name="Aegolism" duration={2.5 * 60 * 60} /> */}
    </div>
  );
}

function BuffButton({
  name,
  duration,
}: {
  name: String;
  duration: number;
}): ReactNode {
  return (
    <p>
      <button
        onClick={() =>
          emit("new-spell-timer", {
            name,
            duration,
            uuid: newUUID(),
          } as SpellTimer)
        }
      >
        {name}
      </button>
    </p>
  );
}

export default MainWindow;
