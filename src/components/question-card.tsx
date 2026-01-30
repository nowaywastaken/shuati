import { useState } from 'react'
import { MarkdownRenderer } from './markdown-renderer'
import { RadioGroup, RadioGroupItem } from './ui/radio-group'
import { Label } from './ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs'
import { cn } from '@/lib/utils'

interface QuestionCardProps {
  question: {
    id: string
    number?: number
    content: string
    options?: string[]
    question_type: string
    difficulty?: number
  }
  onAnswer?: (answer: string, isCorrect: boolean) => void
  showExplanation?: boolean
  explanation?: string
  correctAnswer?: string
}

export function QuestionCard({
  question,
  onAnswer,
  showExplanation = false,
  explanation,
  correctAnswer,
}: QuestionCardProps) {
  const [selectedAnswer, setSelectedAnswer] = useState<string | null>(null)
  const [hasSubmitted, setHasSubmitted] = useState(false)

  const handleAnswer = (value: string) => {
    if (hasSubmitted) return
    
    setSelectedAnswer(value)
    setHasSubmitted(true)
    onAnswer?.(value, value === correctAnswer)
  }

  const isCorrect = selectedAnswer === correctAnswer

  return (
    <div className="border rounded-lg p-6 space-y-4">
      {/* 题号和难度 */}
      <div className="flex items-center justify-between">
        {question.number && (
          <span className="text-sm text-muted-foreground">
            第 {question.number} 题
          </span>
        )}
        {question.difficulty && (
          <div className="flex gap-0.5">
            {Array.from({ length: 5 }).map((_, i) => (
              <div
                key={i}
                className={cn(
                  "w-2 h-2 rounded-full",
                  i < question.difficulty! ? "bg-primary" : "bg-muted"
                )}
              />
            ))}
          </div>
        )}
      </div>

      {/* 题干 */}
      <MarkdownRenderer
        content={question.content}
        className="text-lg leading-relaxed"
      />

      {/* 选项 - 选择题 */}
      {question.question_type === 'multiple_choice' && question.options && (
        <RadioGroup
          value={selectedAnswer || ''}
          onValueChange={handleAnswer}
          disabled={hasSubmitted}
          className="space-y-3 mt-4"
        >
          {question.options.map((option, index) => {
            const optionLabel = String.fromCharCode(65 + index) // A, B, C, D
            const isSelected = selectedAnswer === optionLabel
            const isCorrectOption = correctAnswer === optionLabel
            
            return (
              <div
                key={optionLabel}
                className={cn(
                  "flex items-center space-x-3 p-3 rounded-lg border transition-all cursor-pointer",
                  hasSubmitted && isCorrectOption && "border-green-500 bg-green-50",
                  hasSubmitted && isSelected && !isCorrectOption && "border-red-500 bg-red-50",
                  !hasSubmitted && "hover:border-primary/50"
                )}
                onClick={() => handleAnswer(optionLabel)}
              >
                <RadioGroupItem value={optionLabel} id={optionLabel} />
                <Label
                  htmlFor={optionLabel}
                  className="flex-1 cursor-pointer"
                >
                  <span className="font-medium mr-2">{optionLabel}.</span>
                  {option}
                </Label>
              </div>
            )
          })}
        </RadioGroup>
      )}

      {/* 解析 Tab */}
      {showExplanation && explanation && (
        <Tabs defaultValue="explanation" className="mt-6">
          <TabsList>
            <TabsTrigger value="explanation">AI 解析</TabsTrigger>
            <TabsTrigger value="history">历史记录</TabsTrigger>
          </TabsList>
          <TabsContent value="explanation">
            <MarkdownRenderer content={explanation} className="mt-2" />
          </TabsContent>
          <TabsContent value="history">
            <p className="text-muted-foreground">暂无历史记录</p>
          </TabsContent>
        </Tabs>
      )}
    </div>
  )
}