import { Line } from "react-konva";
import { GRID_SIZE } from "../geometry";

type Props = {
  width: number;
  height: number;
  offsetX: number;
  offsetY: number;
  zoom: number;
};

export function Grid({ width, height, offsetX, offsetY, zoom }: Props) {
  const stroke = `rgba(255,255,255,${0.5 * zoom})`;

  // Visible world-space rect
  const topLeftX = -offsetX / zoom;
  const topLeftY = -offsetY / zoom;
  const bottomRightX = (width - offsetX) / zoom;
  const bottomRightY = (height - offsetY) / zoom;

  const startX = Math.ceil(topLeftX / GRID_SIZE);
  const endX = Math.floor(bottomRightX / GRID_SIZE);
  const startY = Math.floor(topLeftY / GRID_SIZE);
  const endY = Math.ceil(bottomRightY / GRID_SIZE);

  const lines: React.ReactNode[] = [];

  for (let i = startX; i <= endX; i++) {
    const x = i * GRID_SIZE;
    lines.push(
      <Line
        key={`v${i}`}
        points={[x, topLeftY, x, bottomRightY]}
        stroke={stroke}
        strokeWidth={0.5 / zoom}
        listening={false}
      />,
    );
  }

  for (let i = startY; i <= endY; i++) {
    const y = i * GRID_SIZE;
    lines.push(
      <Line
        key={`h${i}`}
        points={[topLeftX, y, bottomRightX, y]}
        stroke={stroke}
        strokeWidth={0.5 / zoom}
        listening={false}
      />,
    );
  }

  return <>{lines}</>;
}
