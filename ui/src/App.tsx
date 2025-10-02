import { useEffect } from 'react';
import Reader from './pages/Reader';
import { usePlayerStore } from './store/playerStore';

const App = () => {
  const theme = usePlayerStore((state) => state.preferences.theme);

  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [theme]);

  return <Reader />;
};

export default App;
