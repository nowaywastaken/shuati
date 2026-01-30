import { useState, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/api/dialog'
import { ProblemSet, Problem, Difficulty } from '@/types'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { Label } from './ui/label'
import { Progress } from './ui/progress'
import { cn } from '@/lib/utils'

interface ImportDialogProps {
  open: boolean
  onClose: () => void
  onImportComplete: (problemSet: ProblemSet) => void
}

type ImportStatus = 'idle' | 'parsing' | 'creating' | 'importing' | 'success' | 'error'

interface ParsedQuestion {
  id: string
  number?: number
  question_type: string
  content: string
  options: string[]
  answer?: string
  explanation?: string
  difficulty?: number
  knowledge_tags: string[]
}

interface ParsedDocument {
  title: string
  questions: ParsedQuestion[]
  total_questions: number
  source_path?: string
  parsed_at: string
}

export function ImportDialog({ open, onClose, onImportComplete }: ImportDialogProps) {
  const [status, setStatus] = useState<ImportStatus>('idle')
  const [selectedFile, setSelectedFile] = useState<string | null>(null)
  const [fileName, setFileName] = useState<string>('')
  const [problemSetName, setProblemSetName] = useState('')
  const [description, setDescription] = useState('')
  const [progress, setProgress] = useState(0)
  const [errorMessage, setErrorMessage] = useState('')
  const [importResult, setImportResult] = useState<{
    problemSetName: string
    questionCount: number
  } | null>(null)
  const [isDragging, setIsDragging] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const resetForm = useCallback(() => {
    setSelectedFile(null)
    setFileName('')
    setProblemSetName('')
    setDescription('')
    setProgress(0)
    setErrorMessage('')
    setImportResult(null)
    setStatus('idle')
  }, [])

  const handleClose = useCallback(() => {
    resetForm()
    onClose()
  }, [onClose, resetForm])

  const handleFileSelect = useCallback(async (filePath: string | null) => {
    if (!filePath) return

    try {
      const name = filePath.split(/[/\\]/).pop() || '未知文件'
      setSelectedFile(filePath)
      setFileName(name)
      
      // 自动填充题集名称（去掉扩展名）
      const baseName = name.replace(/\.(md|markdown)$/i, '')
      setProblemSetName(baseName)
    } catch (error) {
      console.error('选择文件失败:', error)
      setErrorMessage('选择文件失败')
    }
  }, [])

  const handleOpenFile = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: 'Markdown 文件', extensions: ['md', 'markdown'] },
          { name: '所有文件', extensions: ['*'] },
        ],
      })
      await handleFileSelect(selected as string | null)
    } catch (error) {
      console.error('打开文件选择器失败:', error)
      setErrorMessage('打开文件选择器失败')
    }
  }, [handleFileSelect])

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)

    const files = e.dataTransfer.files
    if (files.length > 0) {
      const file = files[0]
      if (file.name.match(/\.(md|markdown)$/i)) {
        // Tauri 桌面应用可以使用文件路径
        const filePath = (file as any).path || file.webkitRelativePath
        await handleFileSelect(filePath)
      } else {
        setErrorMessage('请选择 Markdown 文件（.md 或 .markdown）')
      }
    }
  }, [handleFileSelect])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)
  }, [])

  const handleImport = useCallback(async () => {
    if (!selectedFile || !problemSetName.trim()) {
      setErrorMessage('请选择文件并输入题集名称')
      return
    }

    try {
      setStatus('parsing')
      setProgress(0)
      setErrorMessage('')

      // 1. 解析 Markdown 文件
      const parsedDoc = await invoke<ParsedDocument>('extract_questions_from_file', {
        filePath: selectedFile,
      })

      if (!parsedDoc || parsedDoc.questions.length === 0) {
        setStatus('error')
        setErrorMessage('未能从文件中解析出题目')
        return
      }

      setStatus('creating')
      setProgress(20)

      // 2. 创建题集
      const problemSetId = await invoke<string>('create_problem_set', {
        title: problemSetName.trim(),
        description: description.trim() || undefined,
        filePath: selectedFile,
      })

      setStatus('importing')
      setProgress(40)

      // 3. 批量创建题目
      const questions = parsedDoc.questions.map((q) => ({
        id: undefined as unknown as string,
        number: q.number,
        question_type: q.question_type,
        content: q.content,
        options: q.options.join('\n'),
        answer: q.answer || undefined,
        explanation: q.explanation || undefined,
        difficulty: q.difficulty || undefined,
        knowledge_tags: q.knowledge_tags.join(','),
        source_path: selectedFile,
      }))

      // 分批导入以显示进度
      const batchSize = 10
      const totalBatches = Math.ceil(questions.length / batchSize)

      for (let i = 0; i < questions.length; i += batchSize) {
        const batch = questions.slice(i, i + batchSize)
        await invoke('add_problems', {
          setId: problemSetId,
          questions: batch,
        })

        const currentBatch = Math.floor(i / batchSize) + 1
        const batchProgress = 40 + (currentBatch / totalBatches) * 60
        setProgress(Math.min(batchProgress, 99))
      }

      // 4. 获取创建的题集信息
      const problemSet = await invoke<ProblemSet>('get_problem_set', {
        id: problemSetId,
      })

      if (problemSet) {
        setStatus('success')
        setProgress(100)
        setImportResult({
          problemSetName: problemSet.name,
          questionCount: parsedDoc.questions.length,
        })
      } else {
        setStatus('error')
        setErrorMessage('获取题集信息失败')
      }
    } catch (error) {
      console.error('导入失败:', error)
      setStatus('error')
      setErrorMessage(error instanceof Error ? error.message : '导入失败，请重试')
    }
  }, [selectedFile, problemSetName, description])

  const handleAnotherImport = useCallback(() => {
    resetForm()
  }, [resetForm])

  const getStatusText = () => {
    switch (status) {
      case 'parsing':
        return '正在解析 Markdown 文件...'
      case 'creating':
        return '正在创建题集...'
      case 'importing':
        return '正在导入题目...'
      case 'success':
        return '导入完成！'
      case 'error':
        return '导入失败'
      default:
        return ''
    }
  }

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* 遮罩层 */}
      <div 
        className="absolute inset-0 bg-black/50" 
        onClick={handleClose}
      />

      {/* 对话框内容 */}
      <div className="relative bg-background rounded-lg shadow-lg w-full max-w-lg mx-4 max-h-[90vh] overflow-y-auto">
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-6 py-4 border-b">
          <h2 className="text-lg font-semibold">导入题库</h2>
          <button
            onClick={handleClose}
            className="text-muted-foreground hover:text-foreground transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* 内容区域 */}
        <div className="p-6 space-y-4">
          {status === 'success' && importResult ? (
            // 成功状态
            <div className="flex flex-col items-center py-8 text-center">
              <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mb-4">
                <svg className="w-8 h-8 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">导入成功！</h3>
              <p className="text-muted-foreground mb-1">题集「{importResult.problemSetName}」</p>
              <p className="text-muted-foreground">已成功导入 {importResult.questionCount} 道题目</p>
            </div>
          ) : status === 'error' ? (
            // 错误状态
            <div className="flex flex-col items-center py-8 text-center">
              <div className="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mb-4">
                <svg className="w-8 h-8 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">导入失败</h3>
              <p className="text-destructive mb-4">{errorMessage}</p>
              <Button onClick={() => setStatus('idle')}>重新尝试</Button>
            </div>
          ) : status !== 'idle' && status !== 'success' && status !== 'error' ? (
            // 进度状态
            <div className="py-8">
              <div className="flex flex-col items-center mb-4">
                <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-primary mb-4"></div>
                <p className="text-center font-medium">{getStatusText()}</p>
              </div>
              <Progress value={progress} className="h-2" />
              <p className="text-center text-sm text-muted-foreground mt-2">{progress}%</p>
            </div>
          ) : (
            // 表单状态
            <>
              {/* 拖拽区域 */}
              <div
                className={cn(
                  "border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer",
                  isDragging ? "border-primary bg-primary/5" : "border-muted-foreground/25 hover:border-primary/50",
                  selectedFile ? "bg-muted/50" : ""
                )}
                onDrop={handleDrop}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                onClick={handleOpenFile}
              >
                {selectedFile ? (
                  <div className="flex items-center justify-center gap-2 text-sm">
                    <svg className="w-5 h-5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                    <span className="font-medium">{fileName}</span>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        setSelectedFile(null)
                        setFileName('')
                      }}
                      className="text-muted-foreground hover:text-destructive"
                    >
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
                ) : (
                  <>
                    <svg className="w-10 h-10 mx-auto text-muted-foreground mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                    </svg>
                    <p className="text-sm font-medium mb-1">点击选择或拖拽 Markdown 文件到此处</p>
                    <p className="text-xs text-muted-foreground">支持 .md 和 .markdown 格式</p>
                  </>
                )}
              </div>

              {/* 题集名称 */}
              <div className="space-y-2">
                <Label htmlFor="problemSetName">题集名称 <span className="text-destructive">*</span></Label>
                <Input
                  id="problemSetName"
                  placeholder="输入题集名称"
                  value={problemSetName}
                  onChange={(e) => setProblemSetName(e.target.value)}
                  disabled={status !== 'idle'}
                />
              </div>

              {/* 题集描述 */}
              <div className="space-y-2">
                <Label htmlFor="description">题集描述</Label>
                <Input
                  id="description"
                  placeholder="输入题集描述（可选）"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  disabled={status !== 'idle'}
                />
              </div>

              {/* 错误提示 */}
              {errorMessage && status === 'idle' && (
                <p className="text-sm text-destructive">{errorMessage}</p>
              )}

              {/* 操作按钮 */}
              <div className="flex justify-end gap-3 pt-4">
                <Button variant="outline" onClick={handleClose} disabled={status !== 'idle'}>
                  取消
                </Button>
                <Button 
                  onClick={handleImport} 
                  disabled={!selectedFile || !problemSetName.trim() || status !== 'idle'}
                >
                  开始导入
                </Button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  )
}
