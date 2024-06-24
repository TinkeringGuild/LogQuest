import "./DynamicContainer.css";
import { ReactNode } from "react";
import { Rnd } from "react-rnd";

interface DynamicContainerProps {
  x: number;
  y: number;
  width: number;
  height: number;
  children: ReactNode;
}

const DynamicContainer = ({
  width,
  height,
  x,
  y,
  children,
}: DynamicContainerProps) => {
  return (
    <Rnd className="dynamic-container" default={{ width, height, x, y }}>
      {children}
    </Rnd>
  );
};

export default DynamicContainer;
