import { createFileRoute, Link } from '@tanstack/react-router'
import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Loader2 } from 'lucide-react'
import { apiClient, type Stats, type Task } from '@/lib/api/client'
import { TaskStatusBadge } from '@/components/TaskStatusBadge'

export const Route = createFileRoute('/')({
  component: Dashboard,
})

function Dashboard() {
  const [stats, setStats] = useState<Stats | null>(null)
  const [recentTasks, setRecentTasks] = useState<Task[]>([])
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [statsData, tasksData] = await Promise.all([
          apiClient.getStats(),
          apiClient.getTasks()
        ])
        setStats(statsData)
        setRecentTasks(tasksData.slice(0, 5)) // 최근 5개
      } catch (error) {
        console.error('Failed to fetch dashboard data:', error)
      } finally {
        setIsLoading(false)
      }
    }

    fetchData()
  }, [])

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">대시보드</h1>
        <p className="text-muted-foreground">AutoDev 태스크 관리 시스템</p>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
        </div>
      ) : (
        <>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">전체 태스크</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stats?.total_tasks ?? 0}</div>
                <p className="text-xs text-muted-foreground">생성된 태스크</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">진행 중</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stats?.in_progress_tasks ?? 0}</div>
                <p className="text-xs text-muted-foreground">실행 중인 태스크</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">완료</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stats?.completed_tasks ?? 0}</div>
                <p className="text-xs text-muted-foreground">완료된 태스크</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">실패</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stats?.failed_tasks ?? 0}</div>
                <p className="text-xs text-muted-foreground">실패한 태스크</p>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>최근 태스크</CardTitle>
              <CardDescription>가장 최근에 생성된 태스크 목록</CardDescription>
            </CardHeader>
            <CardContent>
              {recentTasks.length > 0 ? (
                <div className="space-y-4">
                  {recentTasks.map((task) => (
                    <Link
                      key={task.id}
                      to="/tasks/$taskId"
                      params={{ taskId: task.id }}
                      className="flex items-center justify-between p-3 rounded-lg border hover:bg-accent transition-colors"
                    >
                      <div className="flex-1 min-w-0">
                        <div className="font-medium truncate">{task.title}</div>
                        <div className="text-sm text-muted-foreground">
                          {new Date(task.created_at).toLocaleString('ko-KR')}
                        </div>
                      </div>
                      <TaskStatusBadge status={task.status} />
                    </Link>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">아직 태스크가 없습니다.</p>
              )}
            </CardContent>
          </Card>
        </>
      )}
    </div>
  )
}
