import { useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui';
import { DecorativeBlob } from '@/components/common';
import { IconBack } from '@/components/icons';
import { IMAGES } from '@/constants/images';

export function NotFoundPage() {
  const navigate = useNavigate();

  return (
    <div className="relative flex min-h-[60vh] flex-col items-center justify-center text-center overflow-hidden">
      <DecorativeBlob color="#b8a4ed" size={200} x="20%" y="10%" opacity={0.08} />
      <DecorativeBlob color="#ffb084" size={160} x="70%" y="60%" opacity={0.06} />

      <div className="relative z-10">
        <img
          src={IMAGES.notFound}
          alt="Page not found"
          width={320}
          height={240}
          className="rounded-xl mx-auto"
        />
        <h1 className="mt-6 text-4xl font-bold text-ink">Page Not Found</h1>
        <p className="mt-3 text-lg text-body">
          The page you are looking for does not exist or has been moved.
        </p>
        <Button
          variant="primary"
          className="mt-6"
          onClick={() => navigate('/')}
        >
          <IconBack size={16} className="mr-1.5" />
          Back to Home
        </Button>
      </div>
    </div>
  );
}
