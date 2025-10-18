import { createFileRoute } from '@tanstack/react-router'
import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Loader2, ExternalLink } from 'lucide-react'
import { apiClient, type Task, type CompositeTask } from '@/lib/api/client'
import { TaskStatusBadge } from '@/components/TaskStatusBadge'

export const Route = createFileRoute('/tasks/$taskId')({
  component: TaskDetail,
})

type TaskData = {
  type: 'simple'
  task: Task
} | {
  type: 'composite'
  task: CompositeTask
}

function TaskDetail() {
  const { taskId } = Route.useParams()
  const [taskData, setTaskData] = useState<TaskData | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // 태스크 데이터 가져오기
  const fetchTaskData = async () => {
    try {
      // 먼저 단순 태스크로 시도
      try {
        const task = await apiClient.getTask(taskId)
        setTaskData({ type: 'simple', task })
        setIsLoading(false)
        return
      } catch {
        // 단순 태스크가 아니면 복합 태스크로 시도
        const compositeTask = await apiClient.getCompositeTask(taskId)
        setTaskData({ type: 'composite', task: compositeTask })
        setIsLoading(false)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '태스크를 불러오는데 실패했습니다')
      setIsLoading(false)
    }
  }

  // 초기 로딩
  useEffect(() => {
    fetchTaskData()
  }, [taskId])

  // 2초마다 폴링 (진행 중인 태스크가 있는 경우에만)
  useEffect(() => {
    if (!taskData) return

    const hasInProgressTask = () => {
      if (taskData.type === 'simple') {
        return taskData.task.status === 'InProgress' || taskData.task.status === 'Pending' || taskData.task.status === 'Ready'
      } else {
        return taskData.task.subtasks.some(st =>
          st.status === 'InProgress' || st.status === 'Pending' || st.status === 'Ready' || st.status === 'WaitingDependencies'
        )
      }
    }

    if (!hasInProgressTask()) return

    const interval = setInterval(() => {
      fetchTaskData()
    }, 2000)

    return () => clearInterval(interval)
  }, [taskData, taskId])

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-96">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (error || !taskData) {
    return (
      <div className="space-y-6">
        <h1 className="text-3xl font-bold">태스크 상세</h1>
        <Card>
          <CardContent className="py-8">
            <p className="text-center text-destructive">{error || '태스크를 찾을 수 없습니다'}</p>
          </CardContent>
        </Card>
      </div>
    )
  }

  if (taskData.type === 'simple') {
    const task = taskData.task
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">{task.title}</h1>
            <p className="text-muted-foreground">태스크 ID: {taskId}</p>
          </div>
          <TaskStatusBadge status={task.status} />
        </div>

        <div className="grid gap-6 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>기본 정보</CardTitle>
              <CardDescription>태스크의 기본 정보</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <div className="text-sm font-medium text-muted-foreground">제목</div>
                <div className="text-lg">{task.title}</div>
              </div>
              <div>
                <div className="text-sm font-medium text-muted-foreground">설명</div>
                <div>{task.description}</div>
              </div>
              <div>
                <div className="text-sm font-medium text-muted-foreground">타입</div>
                <div>{task.task_type}</div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>실행 정보</CardTitle>
              <CardDescription>태스크 실행 관련 정보</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <div className="text-sm font-medium text-muted-foreground">생성일</div>
                <div>{new Date(task.created_at).toLocaleString('ko-KR')}</div>
              </div>
              <div>
                <div className="text-sm font-medium text-muted-foreground">시작일</div>
                <div>{task.started_at ? new Date(task.started_at).toLocaleString('ko-KR') : '-'}</div>
              </div>
              <div>
                <div className="text-sm font-medium text-muted-foreground">완료일</div>
                <div>{task.completed_at ? new Date(task.completed_at).toLocaleString('ko-KR') : '-'}</div>
              </div>
              {task.pr_url && (
                <div>
                  <div className="text-sm font-medium text-muted-foreground">Pull Request</div>
                  <a
                    href={task.pr_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary hover:underline flex items-center gap-1"
                  >
                    PR 보기 <ExternalLink className="w-3 h-3" />
                  </a>
                </div>
              )}
              {task.error && (
                <div>
                  <div className="text-sm font-medium text-destructive">에러</div>
                  <div className="text-sm text-destructive">{task.error}</div>
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>프롬프트</CardTitle>
            <CardDescription>태스크 실행에 사용되는 프롬프트</CardDescription>
          </CardHeader>
          <CardContent>
            <pre className="text-sm bg-muted p-4 rounded-lg whitespace-pre-wrap">{task.prompt}</pre>
          </CardContent>
        </Card>
      </div>
    )
  } else {
    const compositeTask = taskData.task
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">{compositeTask.title}</h1>
            <p className="text-muted-foreground">복합 태스크 ID: {taskId}</p>
          </div>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>하위 태스크</CardTitle>
            <CardDescription>배치별로 그룹화된 하위 태스크들</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            {compositeTask.batches.map((batch, batchIndex) => (
              <div key={batchIndex} className="space-y-3">
                <h3 className="text-sm font-semibold text-muted-foreground">
                  배치 {batchIndex + 1} ({batch.length}개 태스크)
                </h3>
                <div className="space-y-2">
                  {batch.map((taskId) => {
                    const task = compositeTask.subtasks.find(t => t.id === taskId)
                    if (!task) return null

                    return (
                      <div
                        key={task.id}
                        className="flex items-center justify-between p-3 rounded-lg border bg-card hover:bg-accent/50 transition-colors"
                      >
                        <div className="flex-1 min-w-0">
                          <div className="font-medium truncate">{task.title}</div>
                          <div className="text-sm text-muted-foreground truncate">{task.description}</div>
                        </div>
                        <div className="flex items-center gap-3 ml-4">
                          {task.pr_url && (
                            <a
                              href={task.pr_url}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-primary hover:underline flex items-center gap-1 text-sm"
                            >
                              PR <ExternalLink className="w-3 h-3" />
                            </a>
                          )}
                          <TaskStatusBadge status={task.status} />
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      </div>
    )
  }
}
