// import { listen } from '@tauri-apps/api/event';
// import { useState, useEffect, useReducer } from 'react';
// import { pull } from 'lodash';
/* import { println } from "../util"; */

// import Countdown from './Countdown';
// import DynamicContainer from './DynamicContainer';

/*
interface OverlayDisplay {
  spells: SpellTimer[];
}

interface OverlayAction {
  type: 'spell-added' | 'spell-timer-finished';
  payload: SpellTimer;
}

function reducer(state: OverlayDisplay, action: OverlayAction) {
  switch (action.type) {
    case 'spell-added':
      return { ...state, spells: [...state.spells, action.payload] };
    case 'spell-timer-finished':
      return { ...state, spells: pull(state.spells, action.payload) };
    default:
      throw new Error('unrecognized OverAction!');
  }
}

function OverlayWindow() {
  const [editable, setEditable] = useState(false);
  const [display, dispatch] = useReducer(
    reducer,
    [],
    (spells) =>
      ({
        spells,
      }) as OverlayDisplay
  );

  useEffect(() => {
    let removalTimer: number | null = null;
    const unlisten = listen<SpellTimer>('new-spell-timer', ({ payload }) => {
      dispatch({ type: 'spell-added', payload });
      removalTimer = setTimeout(() => {
        dispatch({ type: 'spell-timer-finished', payload });
      }, payload.duration * 1000);
    });
    return () => {
      unlisten.then((fn) => {
        fn();
        if (removalTimer !== null) {
          clearTimeout(removalTimer);
        }
      });
    };
  }, []);

  useEffect(() => {
    const unlisten = listen('editable-changed', (event) => {
      setEditable(!!event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  return (
    <div className={`overlay ${editable ? 'is-editable' : 'is-static'}`}>
      <DynamicContainer width={450} height={500} x={0} y={0}>
        {display.spells.map((spell) => (
          <Countdown
            label={spell.name}
            duration={spell.duration}
            key={spell.uuid}
          />
        ))}
      </DynamicContainer>
    </div>
  );
}
*/

const OverlayWindow: React.FC<{}> = () => <div>WIP</div>;

export default OverlayWindow;
