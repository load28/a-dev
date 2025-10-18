import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'

export const Route = createFileRoute('/tasks/')({
  component: TaskList,
})

function TaskList() {
  const navigate = useNavigate()

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
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground">
                  태스크가 없습니다
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
