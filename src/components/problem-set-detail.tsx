import { useEffect, useState, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { ProblemSet, Problem, Difficulty, ProgressStatus } from '@/types'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { cn } from '@/lib/utils'

interface ProblemSetDetailProps {
  problemSetId: string
  onBack: () => void
  onStartPractice: (problemSet: ProblemSet) => void
}

type FilterStatus = 'all' | 'not_started' | 'completed' | 'wrong'

export function ProblemSetDetail({
  problemSetId,
  onBack,
  onStartPractice,
}: ProblemSetDetailProps) {
  const [problemSet, setProblemSet] = useState<ProblemSet | null>(null)
  const [problems, setProblems] = useState<Problem[]>([])
  const [loading, setLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState('')
  const [filterStatus, setFilterStatus] = useState<FilterStatus>('all')
  const [userProgress, setUserProgress] = useState<Record<number, ProgressStatus>>({})

  useEffect(() => {
    loadData()
  }, [problemSetId])

  const loadData = async () => {
    try {
      setLoading(true)
      const [setData, problemsData] = await Promise.all([
        invoke<ProblemSet>('get_problem_set_by_id', { id: parseInt(problemSetId) }),
        invoke<Problem[]>('get_problems_by_set_id', { setId: parseInt(problemSetId) }),
      ])
      setProblemSet(setData)
      setProblems(problemsData)
      
      // 加载用户进度
      const progress: Record<number, ProgressStatus> = {}
      for (const problem of problemsData) {
        try {
          const status = await invoke<ProgressStatus>('get_problem_status', { problemId: problem.id })
          progress[problem.id] = status
        } catch {
          progress[problem.id] = ProgressStatus.NotStarted
        }
      }
      setUserProgress(progress)
    } catch (error) {
      console.error('Failed to load data:', error)
    } finally {
      setLoading(false)
    }
  }

  const filteredProblems = useMemo(() => {
    return problems.filter((problem) => {
      // 搜索过滤
      if (searchQuery && !problem.title.toLowerCase().includes(searchQuery.toLowerCase())) {
        return false
      }
      
      // 状态过滤
      const status = userProgress[problem.id] || ProgressStatus.NotStarted
      switch (filterStatus) {
        case 'not_started':
          return status === ProgressStatus.NotStarted
        case 'completed':
          return status === ProgressStatus.Completed
        case 'wrong':
          return status === ProgressStatus.Reviewing
        default:
          return true
      }
    })
  }, [problems, searchQuery, filterStatus, userProgress])

  const getDifficultyColor = (difficulty: Difficulty) => {
    switch (difficulty) {
      case Difficulty.Easy:
        return 'text-green-500 bg-green-50'
      case Difficulty.Medium:
        return 'text-yellow-500 bg-yellow-50'
      case Difficulty.Hard:
        return 'text-red-500 bg-red-50'
      default:
        return 'text-gray-500 bg-gray-50'
    }
  }

  const getStatusIcon = (status: ProgressStatus) => {
    switch (status) {
      case ProgressStatus.Completed:
        return (
          <svg className="w-5 h-5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
          </svg>
        )
      case ProgressStatus.Reviewing:
        return (
          <svg className="w-5 h-5 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        )
      case ProgressStatus.InProgress:
        return (
          <svg className="w-5 h-5 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        )
      default:
        return (
          <svg className="w-5 h-5 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        )
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-3">
          <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-primary"></div>
          <p className="text-muted-foreground">加载中...</p>
        </div>
      </div>
    )
  }

  if (!problemSet) {
    return (
      <div className="flex flex-col items-center justify-center h-64">
        <p className="text-muted-foreground mb-4">题集不存在</p>
        <Button onClick={onBack}>返回</Button>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* 顶部：题集信息和操作按钮 */}
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={onBack}>
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
          </Button>
          <div>
            <h1 className="text-2xl font-bold">{problemSet.name}</h1>
            {problemSet.description && (
              <p className="text-muted-foreground mt-1">{problemSet.description}</p>
            )}
          </div>
        </div>
        <Button onClick={() => onStartPractice(problemSet)}>
          <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          开始刷题
        </Button>
      </div>

      {/* 搜索框和筛选器 */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <svg
            className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <Input
            placeholder="搜索题目..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
        <div className="flex gap-2">
          {[
            { value: 'all', label: '全部' },
            { value: 'not_started', label: '未做' },
            { value: 'completed', label: '已做' },
            { value: 'wrong', label: '错题' },
          ].map((filter) => (
            <Button
              key={filter.value}
              variant={filterStatus === filter.value ? 'default' : 'outline'}
              size="sm"
              onClick={() => setFilterStatus(filter.value as FilterStatus)}
            >
              {filter.label}
            </Button>
          ))}
        </div>
      </div>

      {/* 统计信息 */}
      <div className="flex gap-6 text-sm text-muted-foreground">
        <span>共 {problems.length} 道题目</span>
        <span>
          已完成{' '}
          {Object.values(userProgress).filter((s) => s === ProgressStatus.Completed).length} 道
        </span>
        <span>
          错题{' '}
          {Object.values(userProgress).filter((s) => s === ProgressStatus.Reviewing).length} 道
        </span>
      </div>

      {/* 题目列表 */}
      {filteredProblems.length === 0 ? (
        <div className="flex flex-col items-center justify-center h-64 border-2 border-dashed rounded-lg">
          <p className="text-muted-foreground">
            {searchQuery || filterStatus !== 'all' ? '没有找到匹配的题目' : '暂无题目'}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredProblems.map((problem, index) => {
            const status = userProgress[problem.id] || ProgressStatus.NotStarted
            return (
              <div
                key={problem.id}
                className={cn(
                  "border rounded-lg p-4 cursor-pointer transition-all hover:shadow-md",
                  "bg-card text-card-foreground"
                )}
              >
                {/* 头部：题号和状态 */}
                <div className="flex items-center justify-between mb-3">
                  <span className="text-sm text-muted-foreground">
                    #{index + 1}
                  </span>
                  {getStatusIcon(status)}
                </div>

                {/* 标题 */}
                <h3 className="font-medium truncate mb-2" title={problem.title}>
                  {problem.title}
                </h3>

                {/* 底部：难度和分类 */}
                <div className="flex items-center justify-between">
                  <span
                    className={cn(
                      "text-xs px-2 py-1 rounded-full",
                      getDifficultyColor(problem.difficulty)
                    )}
                  >
                    {problem.difficulty}
                  </span>
                  <span className="text-xs text-muted-foreground">
                    {problem.category}
                  </span>
                </div>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
