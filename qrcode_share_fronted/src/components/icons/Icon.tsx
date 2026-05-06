import { type SVGProps } from 'react';

export interface IconProps extends SVGProps<SVGSVGElement> {
  size?: number;
}

export function createIcon(
  path: string,
  displayName: string,
  options?: { viewBox?: string; fill?: boolean }
) {
  const Component = ({ size = 20, className, ...props }: IconProps) => (
    <svg
      width={size}
      height={size}
      viewBox={options?.viewBox || '0 0 24 24'}
      fill={options?.fill !== false ? 'currentColor' : 'none'}
      stroke={options?.fill === false ? 'currentColor' : undefined}
      strokeWidth={options?.fill === false ? 2 : undefined}
      strokeLinecap={options?.fill === false ? 'round' : undefined}
      strokeLinejoin={options?.fill === false ? 'round' : undefined}
      className={className}
      aria-hidden="true"
      {...props}
    >
      <path d={path} />
    </svg>
  );

  Component.displayName = displayName;
  return Component;
}

export function createIconWithPaths(
  paths: string[],
  displayName: string,
  options?: { viewBox?: string; fill?: boolean }
) {
  const Component = ({ size = 20, className, ...props }: IconProps) => (
    <svg
      width={size}
      height={size}
      viewBox={options?.viewBox || '0 0 24 24'}
      fill={options?.fill !== false ? 'currentColor' : 'none'}
      stroke={options?.fill === false ? 'currentColor' : undefined}
      strokeWidth={options?.fill === false ? 2 : undefined}
      strokeLinecap={options?.fill === false ? 'round' : undefined}
      strokeLinejoin={options?.fill === false ? 'round' : undefined}
      className={className}
      aria-hidden="true"
      {...props}
    >
      {paths.map((d, i) => (
        <path key={i} d={d} />
      ))}
    </svg>
  );

  Component.displayName = displayName;
  return Component;
}
