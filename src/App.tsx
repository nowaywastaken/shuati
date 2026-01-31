import { useEffect, useState } from "react";
import { initDb, Question, batchImportQuestions, getAllQuestions, QuestionAttempt, saveAttempt } from "./lib/db";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { BookOpen, Brain, Settings as SettingsIcon, Upload, FileText, RefreshCw, Play } from "lucide-react";
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import { generateQuestionsFromText } from "./lib/ai";
import { QuestionCard } from "./components/QuestionCard";
import { PracticeMode } from "./components/PracticeMode";
import { Settings } from "./components/Settings";

type View = 'dashboard' | 'practice' | 'settings';

function App() {
  const [loading, setLoading] = useState(true);
  const [questions, setQuestions] = useState<Question[]>([]);
  const [importing, setImporting] = useState(false);
  const [currentView, setCurrentView] = useState<View>('dashboard');
  const [stats, setStats] = useState({
    totalQuestions: 0,
    practiceSessions: 0,
    mistakesCount: 0
  });

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      await initDb();
      const loadedQuestions = await getAllQuestions();
      setQuestions(loadedQuestions);
      setStats(prev => ({
        ...prev,
        totalQuestions: loadedQuestions.length
      }));
      setLoading(false);
    } catch (err) {
      console.error("Failed to load data:", err);
      setLoading(false);
    }
  };

  const handleImport = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Markdown',
          extensions: ['md', 'txt']
        }]
      });

      if (selected && typeof selected === 'string') {
        setImporting(true);
        const content = await readTextFile(selected);
        const generated = await generateQuestionsFromText(content);
        
        // Convert GeneratedQuestion to Question format
        const newQuestions: Question[] = generated.map(g => ({
          question_type: g.question_type,
          stem: g.stem,
          options: g.options ? JSON.stringify(g.options) : null,
          reference_answer: g.reference_answer,
          detailed_analysis: JSON.stringify(g.detailed_analysis),
          media_refs: g.media_refs ? JSON.stringify(g.media_refs) : null,
          knowledge_tags: g.knowledge_tags ? JSON.stringify(g.knowledge_tags) : null,
          difficulty: g.difficulty || null
        }));

        // Save to database
        const result = await batchImportQuestions(newQuestions);
        console.log("Import result:", result);
        
        if (result.success) {
          // Reload questions from database
          await loadData();
        } else {
          console.error("Import errors:", result.errors);
        }
        
        setImporting(false);
      }
    } catch (err) {
      console.error("Import failed:", err);
      setImporting(false);
    }
  };

  const handleAttempt = async (attempt: QuestionAttempt) => {
    try {
      await saveAttempt(attempt);
      // Update stats
      setStats(prev => ({
        ...prev,
        mistakesCount: !attempt.is_correct ? prev.mistakesCount + 1 : prev.mistakesCount
      }));
    } catch (err) {
      console.error("Failed to save attempt:", err);
    }
  };

  const startPractice = () => {
    if (questions.length > 0) {
      setCurrentView('practice');
      setStats(prev => ({
        ...prev,
        practiceSessions: prev.practiceSessions + 1
      }));
    }
  };

  if (loading) return <div className="flex h-screen items-center justify-center">Loading...</div>;

  // Practice Mode View
  if (currentView === 'practice') {
    return (
      <div className="flex h-screen bg-slate-50 dark:bg-slate-950">
        <aside className="w-64 border-r bg-white p-4 dark:bg-slate-900 border-slate-200 dark:border-slate-800 flex flex-col">
          <h1 className="mb-8 text-2xl font-bold text-slate-900 dark:text-white">Shuati AI</h1>
          <nav className="space-y-2 flex-1">
            <Button variant="ghost" className="w-full justify-start" onClick={() => setCurrentView('dashboard')}>
              <BookOpen className="mr-2 h-4 w-4" />
              Library
            </Button>
            <Button variant="default" className="w-full justify-start">
              <Brain className="mr-2 h-4 w-4" />
              Practice
            </Button>
            <Button variant="ghost" className="w-full justify-start" onClick={() => setCurrentView('settings')}>
              <SettingsIcon className="mr-2 h-4 w-4" />
              Settings
            </Button>
          </nav>
        </aside>
        <main className="flex-1 overflow-y-auto">
          <PracticeMode questions={questions} onAttempt={handleAttempt} onExit={() => setCurrentView('dashboard')} />
        </main>
      </div>
    );
  }

  // Settings View
  if (currentView === 'settings') {
    return (
      <div className="flex h-screen bg-slate-50 dark:bg-slate-950">
        <aside className="w-64 border-r bg-white p-4 dark:bg-slate-900 border-slate-200 dark:border-slate-800 flex flex-col">
          <h1 className="mb-8 text-2xl font-bold text-slate-900 dark:text-white">Shuati AI</h1>
          <nav className="space-y-2 flex-1">
            <Button variant="ghost" className="w-full justify-start" onClick={() => setCurrentView('dashboard')}>
              <BookOpen className="mr-2 h-4 w-4" />
              Library
            </Button>
            <Button variant="ghost" className="w-full justify-start" onClick={startPractice} disabled={questions.length === 0}>
              <Brain className="mr-2 h-4 w-4" />
              Practice
            </Button>
            <Button variant="default" className="w-full justify-start">
              <SettingsIcon className="mr-2 h-4 w-4" />
              Settings
            </Button>
          </nav>
        </aside>
        <main className="flex-1 overflow-y-auto">
          <Settings />
        </main>
      </div>
    );
  }

  // Dashboard View (default)
  return (
    <div className="flex h-screen bg-slate-50 dark:bg-slate-950">
      {/* Sidebar */}
      <aside className="w-64 border-r bg-white p-4 dark:bg-slate-900 border-slate-200 dark:border-slate-800 flex flex-col">
        <h1 className="mb-8 text-2xl font-bold text-slate-900 dark:text-white">Shuati AI</h1>
        <nav className="space-y-2 flex-1">
          <Button 
            variant={currentView === 'dashboard' ? 'default' : 'ghost'} 
            className="w-full justify-start"
            onClick={() => setCurrentView('dashboard')}
          >
            <BookOpen className="mr-2 h-4 w-4" />
            Library
          </Button>
          <Button 
            variant='ghost'
            className="w-full justify-start"
            onClick={startPractice}
            disabled={questions.length === 0}
          >
            <Brain className="mr-2 h-4 w-4" />
            Practice
          </Button>
          <Button
            variant={(currentView as View) === 'settings' ? 'default' : 'ghost'}
            className="w-full justify-start"
            onClick={() => setCurrentView('settings')}
          >
            <SettingsIcon className="mr-2 h-4 w-4" />
            Settings
          </Button>
        </nav>
        
        <div className="pt-4 border-t space-y-2">
            <Button className="w-full" onClick={handleImport} disabled={importing}>
                {importing ? "Processing..." : (
                    <>
                        <Upload className="mr-2 h-4 w-4" />
                        Import Questions
                    </>
                )}
            </Button>
            <Button variant="outline" className="w-full" onClick={loadData}>
                <RefreshCw className="mr-2 h-4 w-4" />
                Refresh
            </Button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto p-8">
        <header className="flex justify-between items-center mb-6">
            <h2 className="text-3xl font-bold tracking-tight">Dashboard</h2>
            {questions.length > 0 && (
              <Button onClick={startPractice}>
                <Play className="mr-2 h-4 w-4" />
                Start Practice
              </Button>
            )}
        </header>
        
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 mb-8">
          <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Questions</CardTitle>
              <BookOpen className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
              <div className="text-2xl font-bold">{stats.totalQuestions}</div>
              <p className="text-xs text-muted-foreground">
                {stats.totalQuestions === 0 ? "Import questions to start" : "Ready to practice"}
              </p>
              </CardContent>
          </Card>
          <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Practice Sessions</CardTitle>
              <Brain className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
              <div className="text-2xl font-bold">{stats.practiceSessions}</div>
              <p className="text-xs text-muted-foreground">Started this week</p>
              </CardContent>
          </Card>
          <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Mistakes</CardTitle>
              <FileText className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
              <div className="text-2xl font-bold">{stats.mistakesCount}</div>
              <p className="text-xs text-muted-foreground">To review</p>
              </CardContent>
          </Card>
        </div>
        
        {questions.length === 0 ? (
          <div className="text-center py-12">
            <FileText className="h-12 w-12 mx-auto text-slate-300 mb-4" />
            <h3 className="text-lg font-medium text-slate-600 dark:text-slate-400">No questions yet</h3>
            <p className="text-sm text-slate-500 mt-2 mb-4">Import a Markdown file to generate questions</p>
            <Button onClick={handleImport} disabled={importing}>
              <Upload className="mr-2 h-4 w-4" />
              {importing ? "Processing..." : "Import Questions"}
            </Button>
          </div>
        ) : (
          <div className="max-w-3xl mx-auto">
              <div className="flex items-center gap-2 mb-4 text-slate-500">
                  <FileText className="h-4 w-4" />
                  <span>{questions.length} questions imported</span>
              </div>
              {questions.map(q => (
                  <QuestionCard key={q.id} question={q} />
              ))}
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
