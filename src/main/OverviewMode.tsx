import React from 'react';

const OverviewMode: React.FC<{}> = () => {
  return (
    <div
      className="overview-mode"
      style={{ maxWidth: 250, margin: '20px auto 0' }}
    >
      <img
        width="202"
        height="44"
        src="/LogQuest header black.png"
        alt="LogQuest"
      />
    </div>
  );
};

export default OverviewMode;
