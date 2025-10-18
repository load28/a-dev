import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Button } from '@/components/ui/button'
import { Loader2 } from 'lucide-react'
import { apiClient, type Task } from '@/lib/api/client'
import { TaskStatusBadge } from '@/components/TaskStatusBadge'

export const Route = createFileRoute('/tasks/')({
  component: TaskList,
})

function TaskList() {
  const navigate = useNavigate()
  const [tasks, setTasks] = useState<Task[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const fetchTasks = async () => {
      try {
        const data = await apiClient.getTasks()
        setTasks(data)
      } catch (err) {
        setError(err instanceof Error ? err.message : '태스크 목록을 불러오는데 실패했습니다')
      } finally {
        setIsLoading(false)
      }
    }

    fetchTasks()
  }, [])

  const handleNewTask = () => {
    navigate({ to: '/tasks/new' })
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">태스크 목록</h1>
          <p className="text-muted-foreground">모든 태스크를 확인하고 관리합니다</p>
        </div>
        <Button onClick={handleNewTask}>새 태스크 생성</Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>태스크</CardTitle>
          <CardDescription>생성된 모든 태스크 목록</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
            </div>
          ) : error ? (
            <div className="text-center py-8 text-destructive">{error}</div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>ID</TableHead>
                  <TableHead>제목</TableHead>
                  <TableHead>상태</TableHead>
                  <TableHead>타입</TableHead>
                  <TableHead>생성일</TableHead>
                  <TableHead>액션</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {tasks.length > 0 ? (
                  tasks.map((task) => (
                    <TableRow key={task.id}>
                      <TableCell className="font-mono text-xs">
                        {task.id.slice(0, 8)}...
                      </TableCell>
                      <TableCell className="font-medium">{task.title}</TableCell>
                      <TableCell>
                        <TaskStatusBadge status={task.status} />
                      </TableCell>
                      <TableCell>{task.task_type}</TableCell>
                      <TableCell>
                        {new Date(task.created_at).toLocaleDateString('ko-KR')}
                      </TableCell>
                      <TableCell>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => navigate({ to: `/tasks/${task.id}` })}
                        >
                          상세보기
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell colSpan={6} className="text-center text-muted-foreground">
                      태스크가 없습니다
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
