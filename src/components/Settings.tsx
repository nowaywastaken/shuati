import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Settings as SettingsIcon, Key, Database, Trash2 } from "lucide-react";
import { getOpenAIConfig, configureOpenAI } from "@/lib/ai";

interface SettingsProps {
  className?: string;
}

export function SettingsPage({}: SettingsProps) {
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("gpt-4o");
  const [baseURL, setBaseURL] = useState("https://api.openai.com/v1");
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    const config = getOpenAIConfig();
    if (config) {
      setApiKey(config.apiKey || "");
      setModel(config.model || "gpt-4o");
      setBaseURL(config.baseURL || "https://api.openai.com/v1");
    }
  }, []);

  const handleSave = () => {
    configureOpenAI({
      apiKey: apiKey.trim(),
      model: model.trim() || "gpt-4o",
      baseURL: baseURL.trim() || "https://api.openai.com/v1",
    });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const handleClear = () => {
    setApiKey("");
    configureOpenAI({
      apiKey: "",
      model: "gpt-4o",
      baseURL: "https://api.openai.com/v1",
    });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="max-w-3xl mx-auto p-8">
      <div className="flex items-center gap-2 mb-6">
        <SettingsIcon className="h-6 w-6" />
        <h2 className="text-2xl font-bold">Settings</h2>
      </div>

      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Key className="h-5 w-5" />
              OpenAI API Configuration
            </CardTitle>
            <CardDescription>
              Configure your OpenAI API key to enable AI-powered question generation.
              Your key is stored locally and never sent to external servers other than OpenAI.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="apiKey">API Key</Label>
              <Input
                id="apiKey"
                type="password"
                placeholder="sk-..."
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
              />
              <p className="text-xs text-slate-500">
                Get your API key from{" "}
                <a
                  href="https://platform.openai.com/api-keys"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-600 hover:underline"
                >
                  platform.openai.com
                </a>
              </p>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="model">Model</Label>
                <Input
                  id="model"
                  placeholder="gpt-4o"
                  value={model}
                  onChange={(e) => setModel(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="baseURL">Base URL</Label>
                <Input
                  id="baseURL"
                  placeholder="https://api.openai.com/v1"
                  value={baseURL}
                  onChange={(e) => setBaseURL(e.target.value)}
                />
              </div>
            </div>

            <div className="flex gap-3 pt-2">
              <Button onClick={handleSave} variant="default">
                {saved ? "Saved!" : "Save Configuration"}
              </Button>
              <Button onClick={handleClear} variant="outline">
                <Trash2 className="h-4 w-4 mr-2" />
                Clear
              </Button>
            </div>

            {!apiKey && (
              <div className="p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 rounded-lg">
                <p className="text-sm text-yellow-700 dark:text-yellow-300">
                  No API key configured. Using mock data for question generation.
                </p>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Database className="h-5 w-5" />
              Database Information
            </CardTitle>
            <CardDescription>
              Local SQLite database with WAL mode for optimal performance.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-3 gap-4">
              <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <div className="text-sm text-slate-500 mb-1">Storage Location</div>
                <div className="text-xs font-mono text-slate-700 dark:text-slate-300">
                  ~/Library/Application Support/shuati/
                </div>
              </div>
              <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <div className="text-sm text-slate-500 mb-1">Mode</div>
                <Badge variant="secondary">WAL (Write-Ahead Logging)</Badge>
              </div>
              <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <div className="text-sm text-slate-500 mb-1">Tables</div>
                <div className="flex gap-1 flex-wrap">
                  <Badge variant="outline">questions</Badge>
                  <Badge variant="outline">attempts</Badge>
                  <Badge variant="outline">mistakes</Badge>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <SettingsIcon className="h-5 w-5" />
              Features
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-4">
              <div className="flex items-center gap-3">
                <Badge className="bg-green-500">Active</Badge>
                <span className="text-sm">Markdown Import</span>
              </div>
              <div className="flex items-center gap-3">
                <Badge className="bg-green-500">Active</Badge>
                <span className="text-sm">LaTeX Rendering (KaTeX)</span>
              </div>
              <div className="flex items-center gap-3">
                <Badge className="bg-green-500">Active</Badge>
                <span className="text-sm">Practice Mode</span>
              </div>
              <div className="flex items-center gap-3">
                <Badge className={apiKey ? "bg-green-500" : "bg-yellow-500"}>
                  {apiKey ? "Active" : "Mock"}
                </Badge>
                <span className="text-sm">AI Question Generation</span>
              </div>
              <div className="flex items-center gap-3">
                <Badge variant="outline">Planned</Badge>
                <span className="text-sm">DuckDB Analytics</span>
              </div>
              <div className="flex items-center gap-3">
                <Badge variant="outline">Planned</Badge>
                <span className="text-sm">Spaced Repetition</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export { SettingsPage as Settings };
export type { SettingsProps };
