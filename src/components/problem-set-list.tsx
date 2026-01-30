import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { ProblemSet } from '@/types'
import { Button } from './ui/button'
import { cn } from '@/lib/utils'

interface ProblemSetListProps {
  onSelectProblemSet?: (problemSet: ProblemSet) => void
  onCreateProblemSet?: () => void
  onImportProblemSet?: () => void
}

export function ProblemSetList({
  onSelectProblemSet,
  onCreateProblemSet,
  onImportProblemSet,
}: ProblemSetListProps) {
  const [problemSets, setProblemSets] = useState<ProblemSet[]>([])
  const [loading, setLoading] = useState(true)
  const [deletingId, setDeletingId] = useState<number | null>(null)

  useEffect(() => {
    loadProblemSets()
  }, [])

  const loadProblemSets = async () => {
    try {
      setLoading(true)
      const sets = await invoke<ProblemSet[]>('get_problem_sets')
      setProblemSets(sets)
    } catch (error) {
      console.error('Failed to load problem sets:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleDelete = async (id: number, e: React.MouseEvent) => {
    e.stopPropagation()
    if (!confirm('确定要删除这个题集吗？')) return

    try {
      setDeletingId(id)
      await invoke('delete_problem_set', { id })
      setProblemSets((prev) => prev.filter((s) => s.id !== id))
    } catch (error) {
      console.error('Failed to delete problem set:', error)
    } finally {
      setDeletingId(null)
    }
  }

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    })
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

  return (
    <div className="space-y-6">
      {/* 顶部标题和操作按钮 */}
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">题集列表</h1>
        <div className="flex gap-3">
          <Button variant="outline" onClick={onImportProblemSet}>
            导入题库
          </Button>
          <Button onClick={onCreateProblemSet}>
            新建题集
          </Button>
        </div>
      </div>

      {/* 空状态 */}
      {problemSets.length === 0 ? (
        <div className="flex flex-col items-center justify-center h-64 border-2 border-dashed rounded-lg">
          <div className="text-muted-foreground mb-4">
            <svg
              className="w-16 h-16 mx-auto mb-3"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
              />
            </svg>
            <p className="text-lg">暂无题集</p>
          </div>
          <div className="flex gap-3">
            <Button variant="outline" onClick={onImportProblemSet}>
              导入题库
            </Button>
            <Button onClick={onCreateProblemSet}>
              新建题集
            </Button>
          </div>
        </div>
      ) : (
        /* 题集卡片网格 */
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {problemSets.map((problemSet) => (
            <div
              key={problemSet.id}
              className={cn(
                "border rounded-lg p-5 cursor-pointer transition-all hover:shadow-md",
                "bg-card text-card-foreground"
              )}
              onClick={() => onSelectProblemSet?.(problemSet)}
            >
              {/* 卡片头部 */}
              <div className="flex items-start justify-between mb-3">
                <h3 className="text-lg font-semibold truncate pr-2">
                  {problemSet.name}
                </h3>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 text-muted-foreground hover:text-destructive"
                  onClick={(e) => handleDelete(problemSet.id, e)}
                  disabled={deletingId === problemSet.id}
                >
                  {deletingId === problemSet.id ? (
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-current"></div>
                  ) : (
                    <svg
                      className="w-4 h-4"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                      />
                    </svg>
                  )}
                </Button>
              </div>

              {/* 描述 */}
              {problemSet.description && (
                <p className="text-muted-foreground text-sm mb-4 line-clamp-2">
                  {problemSet.description}
                </p>
              )}

              {/* 统计信息 */}
              <div className="flex items-center gap-4 text-sm text-muted-foreground mb-4">
                <div className="flex items-center gap-1">
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
                    />
                  </svg>
                  <span>{problemSet.problemCount} 道题目</span>
                </div>
                <div className="flex items-center gap-1">
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                    />
                  </svg>
                  <span>{formatDate(problemSet.createdAt)}</span>
                </div>
              </div>

              {/* 操作按钮 */}
              <div className="flex gap-2 pt-3 border-t">
                <Button
                  className="flex-1"
                  size="sm"
                  onClick={(e) => {
                    e.stopPropagation()
                    onSelectProblemSet?.(problemSet)
                  }}
                >
                  开始刷题
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={(e) => {
                    e.stopPropagation()
                    // TODO: 编辑功能
                  }}
                >
                  编辑
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
