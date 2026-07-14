import { Link, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { setAuthToken } from '../api/client';

const navItems = [
  { label: 'Dashboard', path: '/' },
  { label: 'Monitors', path: '/monitors' },
  { label: 'Incidents', path: '/incidents' },
  { label: 'Data Sources', path: '/data-sources' },
  { label: 'Integrations', path: '/integrations' },
  { label: 'Settings', path: '/settings' },
];

export default function Layout() {
  const location = useLocation();
  const navigate = useNavigate();

  const handleLogout = () => {
    setAuthToken(null);
    navigate('/login');
  };

  return (
    <div className="flex h-full">
      <aside className="w-56 bg-gray-900 text-white flex flex-col shrink-0">
        <div className="px-5 py-4 border-b border-gray-700">
          <Link to="/" className="text-lg font-bold tracking-tight">
            OpenQuality
          </Link>
          <div className="text-xs text-gray-400 mt-0.5">Data Quality Platform</div>
        </div>
        <nav className="flex-1 py-3">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={`block px-5 py-2.5 text-sm transition-colors ${
                location.pathname === item.path
                  ? 'bg-brand-600 text-white font-medium'
                  : 'text-gray-300 hover:bg-gray-800 hover:text-white'
              }`}
            >
              {item.label}
            </Link>
          ))}
        </nav>
        <div className="px-5 py-3 border-t border-gray-700">
          <button
            onClick={handleLogout}
            className="text-sm text-gray-400 hover:text-white transition-colors"
          >
            Log out
          </button>
        </div>
      </aside>
      <main className="flex-1 overflow-auto">
        <div className="p-6 max-w-6xl mx-auto">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
