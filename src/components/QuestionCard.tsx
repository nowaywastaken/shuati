import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { MarkdownRenderer } from "./MarkdownRenderer";
import { Question } from "@/lib/db";
import { Lightbulb, BarChart3 } from "lucide-react";

interface QuestionCardProps {
  question: Question;
  showAnalysis?: boolean;
  showAnswer?: boolean;
}

export function QuestionCard({ question, showAnalysis = false, showAnswer = false }: QuestionCardProps) {
  let options = [];
  let tags = [];
  let analysis = [];
  
  try {
    options = question.options ? JSON.parse(question.options) : [];
  } catch (e) {
    console.error("Failed to parse options", e);
  }
  
  try {
    tags = question.knowledge_tags ? JSON.parse(question.knowledge_tags) : [];
  } catch (e) {
    console.error("Failed to parse tags", e);
  }
  
  try {
    analysis = question.detailed_analysis ? JSON.parse(question.detailed_analysis) : [question.detailed_analysis];
  } catch (e) {
    analysis = [question.detailed_analysis];
  }

  const getDifficultyColor = (level: number | null | undefined) => {
    if (!level) return "bg-slate-100 text-slate-700";
    if (level <= 2) return "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300";
    if (level <= 3) return "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-300";
    return "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300";
  };

  return (
    <Card className="mb-4">
      <CardHeader>
        <div className="flex justify-between items-start gap-4">
          <div className="flex items-center gap-2">
            <CardTitle className="text-lg">Question {question.id}</CardTitle>
            <span className="text-xs font-mono bg-slate-100 dark:bg-slate-800 px-2 py-1 rounded text-slate-500 uppercase">
              {question.question_type.replace(/_/g, ' ')}
            </span>
          </div>
          <div className="flex items-center gap-2">
            {question.difficulty && (
              <Badge variant="secondary" className={getDifficultyColor(question.difficulty)}>
                <BarChart3 className="w-3 h-3 mr-1" />
                L{question.difficulty}
              </Badge>
            )}
          </div>
        </div>
        
        {tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-2">
            {tags.map((tag: string, idx: number) => (
              <Badge key={idx} variant="outline" className="text-xs">
                <Lightbulb className="w-3 h-3 mr-1" />
                {tag}
              </Badge>
            ))}
          </div>
        )}
      </CardHeader>
      
      <CardContent className="space-y-4">
        {/* Question Stem */}
        <div className="text-base leading-relaxed">
          <MarkdownRenderer content={question.stem} />
        </div>
        
        {/* Options for Multiple Choice */}
        {options.length > 0 && (
          <div className="grid gap-2 mt-4">
            {options.map((opt: any, idx: number) => {
              const content = typeof opt === 'object' ? opt.content : opt;
              const label = typeof opt === 'object' ? opt.label : String.fromCharCode(65 + idx);
              
              return (
                <div key={idx} className="flex items-start gap-3 p-3 rounded-lg border border-slate-200 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-900 transition-colors cursor-pointer">
                  <div className="flex-shrink-0 w-6 h-6 flex items-center justify-center rounded-full bg-slate-100 dark:bg-slate-800 text-xs font-bold">
                    {label}
                  </div>
                  <div className="pt-0.5">
                    <MarkdownRenderer content={content} />
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {/* Reference Answer */}
        {showAnswer && (
          <div className="mt-4 p-3 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200">
            <p className="text-sm font-medium text-green-700 dark:text-green-300 mb-1">Reference Answer:</p>
            <MarkdownRenderer content={question.reference_answer} />
          </div>
        )}

        {/* Detailed Analysis */}
        {showAnalysis && analysis.length > 0 && (
          <div className="mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200">
            <p className="text-sm font-medium text-blue-700 dark:text-blue-300 mb-2">Analysis:</p>
            <ol className="list-decimal list-inside space-y-1">
              {analysis.map((step: string, idx: number) => (
                <li key={idx} className="text-sm text-slate-600 dark:text-slate-400">
                  <MarkdownRenderer content={step} />
                </li>
              ))}
            </ol>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
