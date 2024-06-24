import "./DynamicContainer.css";
import { ReactNode } from "react";

interface DynamicContainerProps {
  width: number;
  height: number;
  children: ReactNode;
}

const DynamicContainer = ({
  width,
  height,
  children,
}: DynamicContainerProps) => {
  return (
    <div className="dynamic-container" style={{ width, height }}>
      {children}
    </div>
  );
};

export default DynamicContainer;
