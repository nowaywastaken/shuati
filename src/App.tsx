import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './index.css';

interface GreetingResponse {
  message: string;
  version: string;
  platform: string;
}

function App() {
  const [name, setName] = useState('');
  const [greeting, setGreeting] = useState<GreetingResponse | null>(null);
  const [loading, setLoading] = useState(false);

  const handleGreet = async () => {
    if (!name.trim()) return;
    
    setLoading(true);
    try {
      const response = await invoke<GreetingResponse>('greet', { name });
      setGreeting(response);
    } catch (error) {
      console.error('Failed to greet:', error);
      setGreeting({
        message: 'Failed to get greeting',
        version: 'unknown',
        platform: 'unknown',
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-background p-8">
      <div className="max-w-2xl mx-auto">
        <h1 className="text-4xl font-bold text-center mb-8 text-primary">
          🧠 刷题神器
        </h1>
        
        <div className="card">
          <div className="card-body">
            <h2 className="card-title text-2xl mb-4">欢迎使用本地化 AI 刷题应用</h2>
            
            <p className="text-muted-foreground mb-6">
              这是一个基于 Tauri 2.0 构建的跨平台桌面应用，支持：
            </p>
            
            <ul className="list-disc list-inside space-y-2 mb-6 text-muted-foreground">
              <li>本地 Markdown 题库管理</li>
              <li>KaTeX 数学公式渲染</li>
              <li>本地 AI 推理 (llama.cpp)</li>
              <li>SQLite 本地数据库存储</li>
              <li>跨平台支持 (macOS/Windows)</li>
            </ul>

            <div className="join w-full">
              <input
                type="text"
                placeholder="请输入您的名字"
                className="input input-bordered w-full join-item"
                value={name}
                onChange={(e) => setName(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleGreet()}
              />
              <button 
                className="btn btn-primary join-item"
                onClick={handleGreet}
                disabled={loading}
              >
                {loading ? '加载中...' : '问候'}
              </button>
            </div>

            {greeting && (
              <div className="alert alert-success mt-4">
                <svg xmlns="http://www.w3.org/2000/svg" className="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <div>
                  <h3 className="font-bold">{greeting.message}</h3>
                  <div className="text-xs">
                    版本: {greeting.version} | 平台: {greeting.platform}
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
