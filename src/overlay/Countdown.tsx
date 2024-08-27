import './Countdown.css';

interface CountdownProps {
  label: string;
  duration: number;
}

const Countdown: React.FC<CountdownProps> = ({ label, duration }) => {
  return (
    <div className="countdown column-member">
      <div
        className="visual-timer"
        style={{ animationDuration: `${duration / 1000}s` }}
      ></div>
      <p>{label}</p>
    </div>
  );
};

export default Countdown;
