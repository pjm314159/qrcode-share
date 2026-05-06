import { useEffect, useState, useRef, type ReactNode } from 'react';
import { useLocation } from 'react-router-dom';

interface PageTransitionProps {
  children: ReactNode;
}

export function PageTransition({ children }: PageTransitionProps) {
  const location = useLocation();
  const [displayChildren, setDisplayChildren] = useState(children);
  const [prevKey, setPrevKey] = useState(location.key);
  const childrenRef = useRef(children);

  useEffect(() => {
    childrenRef.current = children;
  });

  const transitioning = prevKey !== location.key;

  useEffect(() => {
    if (prevKey === location.key) return;
    const timer = setTimeout(() => {
      setDisplayChildren(childrenRef.current);
      setPrevKey(location.key);
    }, 150);
    return () => clearTimeout(timer);
  }, [location.key, prevKey]);

  return (
    <div
      className={`transition-opacity duration-150 ${
        transitioning ? 'opacity-0' : 'opacity-100'
      }`}
    >
      {displayChildren}
    </div>
  );
}
