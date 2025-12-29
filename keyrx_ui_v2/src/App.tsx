import { useState } from 'react';

function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="min-h-screen bg-bg-primary text-text-primary flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-3xl font-bold mb-8">KeyRX UI v2</h1>
        <div className="mb-4">
          <button
            onClick={() => setCount((count) => count + 1)}
            className="bg-primary-500 hover:bg-primary-600 text-white font-medium py-3 px-4 rounded-md transition-all duration-150"
          >
            count is {count}
          </button>
        </div>
        <p className="text-text-secondary">
          Edit <code className="font-mono">src/App.tsx</code> and save to test
          HMR
        </p>
      </div>
    </div>
  );
}

export default App;
