import { Link } from 'react-router-dom';

export default function NotFound() {
  return (
    <div className="flex flex-col items-center justify-center h-full text-center">
      <h1 className="text-6xl font-bold text-gray-200 mb-4">404</h1>
      <p className="text-gray-500 mb-6">Page not found</p>
      <Link
        to="/"
        className="text-brand-600 hover:text-brand-800 text-sm font-medium"
      >
        Back to Dashboard
      </Link>
    </div>
  );
}
