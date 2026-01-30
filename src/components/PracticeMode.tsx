import { useState, useCallback } from "react";
import { Question, QuestionAttempt } from "@/lib/db";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { MarkdownRenderer } from "./MarkdownRenderer";
import { ArrowRight, ArrowLeft, Check, X, Clock, Flag } from "lucide-react";

interface PracticeModeProps {
  questions: Question[];
  onAttempt: (attempt: QuestionAttempt) => void;
  onExit: () => void;
}

export function PracticeMode({ questions, onAttempt, onExit }: PracticeModeProps) {
  const [currentIndex, setCurrentIndex] = useState(0);
  const [selectedOption, setSelectedOption] = useState<string | null>(null);
  const [textAnswer, setTextAnswer] = useState("");
  const [showAnalysis, setShowAnalysis] = useState(false);
  const [attempts, setAttempts] = useState<QuestionAttempt[]>([]);
  const [startTime] = useState(Date.now());
  const [isFinished, setIsFinished] = useState(false);

  const currentQuestion = questions[currentIndex];
  const isLastQuestion = currentIndex === questions.length - 1;

  const getOptions = () => {
    try {
      if (!currentQuestion.options) return [];
      const parsed = JSON.parse(currentQuestion.options);
      return parsed || [];
    } catch {
      return [];
    }
  };

  const getAnalysis = () => {
    try {
      return JSON.parse(currentQuestion.detailed_analysis) || [];
    } catch {
      return [currentQuestion.detailed_analysis];
    }
  };

  const checkAnswer = useCallback(() => {
    let isCorrect = false;
    let userAnswer = "";

    if (currentQuestion.question_type === 'multiple_choice') {
      isCorrect = selectedOption === currentQuestion.reference_answer;
      userAnswer = selectedOption || '';
    } else if (currentQuestion.question_type === 'fill_in_the_blank') {
      // Simple string comparison (case insensitive)
      isCorrect = textAnswer.toLowerCase().trim() === currentQuestion.reference_answer.toLowerCase().trim();
      userAnswer = textAnswer;
    } else {
      // Essay - would need AI grading
      userAnswer = textAnswer;
      isCorrect = false; // Mark as incorrect for manual review
    }

    const timeSpent = Math.floor((Date.now() - startTime) / 1000);
    
    const attempt: QuestionAttempt = {
      question_id: currentQuestion.id!,
      user_answer: userAnswer,
      is_correct: isCorrect,
      time_spent_seconds: timeSpent,
    };

    setAttempts(prev => [...prev, attempt]);
    onAttempt(attempt);
    setShowAnalysis(true);
  }, [currentQuestion, selectedOption, textAnswer, startTime, onAttempt]);

  const handleNext = () => {
    if (isLastQuestion) {
      setIsFinished(true);
    } else {
      setCurrentIndex(prev => prev + 1);
      setSelectedOption(null);
      setTextAnswer("");
      setShowAnalysis(false);
    }
  };

  const handlePrevious = () => {
    if (currentIndex > 0) {
      setCurrentIndex(prev => prev - 1);
      setSelectedOption(null);
      setTextAnswer("");
      setShowAnalysis(false);
    }
  };

  // Summary view
  if (isFinished) {
    const correctCount = attempts.filter(a => a.is_correct).length;
    const accuracy = Math.round((correctCount / attempts.length) * 100) || 0;

    return (
      <div className="max-w-2xl mx-auto p-8">
        <Card>
          <CardHeader>
            <CardTitle className="text-center">Practice Session Complete!</CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="grid grid-cols-3 gap-4 text-center">
              <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <div className="text-3xl font-bold">{questions.length}</div>
                <div className="text-sm text-slate-500">Questions</div>
              </div>
              <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded-lg">
                <div className="text-3xl font-bold text-green-600">{correctCount}</div>
                <div className="text-sm text-slate-500">Correct</div>
              </div>
              <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                <div className="text-3xl font-bold text-blue-600">{accuracy}%</div>
                <div className="text-sm text-slate-500">Accuracy</div>
              </div>
            </div>
            
            <div className="flex gap-4 justify-center">
              <Button variant="outline" onClick={onExit}>
                Back to Dashboard
              </Button>
              <Button onClick={() => window.location.reload()}>
                Practice Again
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  const options = getOptions();
  const analysis = getAnalysis();

  return (
    <div className="max-w-3xl mx-auto p-8">
      {/* Progress Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <Button variant="outline" size="sm" onClick={onExit}>
            Exit
          </Button>
          <span className="text-sm text-slate-500">
            Question {currentIndex + 1} of {questions.length}
          </span>
        </div>
        <div className="flex items-center gap-2 text-sm text-slate-500">
          <Clock className="h-4 w-4" />
          <span>{Math.floor((Date.now() - startTime) / 1000)}s</span>
        </div>
      </div>

      {/* Progress Bar */}
      <div className="w-full bg-slate-200 dark:bg-slate-700 h-2 rounded-full mb-6">
        <div 
          className="bg-blue-600 h-2 rounded-full transition-all"
          style={{ width: `${((currentIndex + 1) / questions.length) * 100}%` }}
        />
      </div>

      {/* Question Card */}
      <Card className="mb-6">
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle>Question {currentQuestion.id}</CardTitle>
            <div className="flex gap-2">
              <span className="text-xs font-mono bg-slate-100 dark:bg-slate-800 px-2 py-1 rounded text-slate-500 uppercase">
                {currentQuestion.question_type}
              </span>
              {currentQuestion.difficulty && (
                <span className="text-xs font-mono bg-yellow-100 dark:bg-yellow-900/30 px-2 py-1 rounded text-yellow-700 dark:text-yellow-300">
                  L{currentQuestion.difficulty}
                </span>
              )}
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {/* Question Stem */}
          <div className="mb-6 text-lg">
            <MarkdownRenderer content={currentQuestion.stem} />
          </div>

          {/* Answer Input */}
          {!showAnalysis && (
            <>
              {currentQuestion.question_type === 'multiple_choice' && options.length > 0 && (
                <div className="grid gap-3">
                  {options.map((opt: any, idx: number) => {
                    const content = typeof opt === 'object' ? opt.content : opt;
                    const label = typeof opt === 'object' ? opt.label : String.fromCharCode(65 + idx);
                    const isSelected = selectedOption === label;
                    
                    return (
                      <button
                        key={idx}
                        onClick={() => setSelectedOption(label)}
                        className={`flex items-start gap-3 p-4 rounded-lg border-2 text-left transition-all ${
                          isSelected 
                            ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' 
                            : 'border-slate-200 dark:border-slate-700 hover:border-slate-300'
                        }`}
                      >
                        <div className={`flex-shrink-0 w-8 h-8 flex items-center justify-center rounded-full font-bold ${
                          isSelected 
                            ? 'bg-blue-500 text-white' 
                            : 'bg-slate-100 dark:bg-slate-800'
                        }`}>
                          {label}
                        </div>
                        <div className="pt-1">
                          <MarkdownRenderer content={content} />
                        </div>
                      </button>
                    );
                  })}
                </div>
              )}

              {(currentQuestion.question_type === 'fill_in_the_blank' || currentQuestion.question_type === 'essay') && (
                <div className="space-y-4">
                  <textarea
                    value={textAnswer}
                    onChange={(e) => setTextAnswer(e.target.value)}
                    placeholder={currentQuestion.question_type === 'essay' ? "Write your answer here..." : "Enter your answer"}
                    className="w-full min-h-[120px] p-4 border-2 border-slate-200 dark:border-slate-700 rounded-lg focus:border-blue-500 focus:outline-none bg-transparent"
                  />
                </div>
              )}
            </>
          )}

          {/* Analysis */}
          {showAnalysis && (
            <div className="space-y-4">
              <div className={`p-4 rounded-lg ${
                attempts[attempts.length - 1]?.is_correct 
                  ? 'bg-green-50 dark:bg-green-900/20 border border-green-200' 
                  : 'bg-red-50 dark:bg-red-900/20 border border-red-200'
              }`}>
                <div className="flex items-center gap-2 mb-2">
                  {attempts[attempts.length - 1]?.is_correct ? (
                    <>
                      <Check className="h-5 w-5 text-green-600" />
                      <span className="font-medium text-green-700 dark:text-green-300">Correct!</span>
                    </>
                  ) : (
                    <>
                      <X className="h-5 w-5 text-red-600" />
                      <span className="font-medium text-red-700 dark:text-red-300">Incorrect</span>
                    </>
                  )}
                </div>
                <div className="text-sm">
                  <span className="text-slate-500">Your answer: </span>
                  <span className={attempts[attempts.length - 1]?.is_correct ? 'text-green-700' : 'text-red-700'}>
                    {attempts[attempts.length - 1]?.user_answer || '(empty)'}
                  </span>
                </div>
                <div className="text-sm mt-1">
                  <span className="text-slate-500">Correct answer: </span>
                  <span className="text-green-700 font-medium">
                    <MarkdownRenderer content={currentQuestion.reference_answer} />
                  </span>
                </div>
              </div>

              <div className="bg-slate-50 dark:bg-slate-800 p-4 rounded-lg">
                <h4 className="font-medium mb-3 flex items-center gap-2">
                  <Flag className="h-4 w-4" />
                  Detailed Analysis
                </h4>
                <ol className="space-y-2 list-decimal list-inside">
                  {analysis.map((step: string, idx: number) => (
                    <li key={idx} className="text-sm text-slate-600 dark:text-slate-400">
                      <MarkdownRenderer content={step} />
                    </li>
                  ))}
                </ol>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Navigation */}
      <div className="flex justify-between">
        <Button
          variant="outline"
          onClick={handlePrevious}
          disabled={currentIndex === 0}
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          Previous
        </Button>

        {!showAnalysis ? (
          <Button
            onClick={checkAnswer}
            disabled={
              (currentQuestion.question_type === 'multiple_choice' && !selectedOption) ||
              ((currentQuestion.question_type === 'fill_in_the_blank' || currentQuestion.question_type === 'essay') && !textAnswer.trim())
            }
          >
            Submit Answer
          </Button>
        ) : (
          <Button onClick={handleNext}>
            {isLastQuestion ? 'Finish' : (
              <>
                Next
                <ArrowRight className="ml-2 h-4 w-4" />
              </>
            )}
          </Button>
        )}
      </div>
    </div>
  );
}
