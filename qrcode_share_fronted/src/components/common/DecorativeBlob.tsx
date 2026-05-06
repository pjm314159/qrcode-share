interface DecorativeBlobProps {
  color: string;
  size?: number;
  x: string;
  y: string;
  opacity?: number;
}

export function DecorativeBlob({ color, size = 300, x, y, opacity = 0.15 }: DecorativeBlobProps) {
  return (
    <div
      className="absolute rounded-full pointer-events-none"
      style={{
        width: size,
        height: size,
        left: x,
        top: y,
        backgroundColor: color,
        opacity,
        filter: 'blur(60px)',
      }}
      aria-hidden="true"
    />
  );
}
