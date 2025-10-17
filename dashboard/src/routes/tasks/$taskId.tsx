import { createFileRoute } from '@tanstack/react-router'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'

export const Route = createFileRoute('/tasks/$taskId')({
  component: TaskDetail,
})

function TaskDetail() {
  const { taskId } = Route.useParams()

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">태스크 상세</h1>
          <p className="text-muted-foreground">태스크 ID: {taskId}</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline">삭제</Button>
          <Button>실행</Button>
        </div>
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
              <div className="text-lg">태스크 제목</div>
            </div>
            <div>
              <div className="text-sm font-medium text-muted-foreground">설명</div>
              <div>태스크 설명</div>
            </div>
            <div>
              <div className="text-sm font-medium text-muted-foreground">상태</div>
              <Badge>Pending</Badge>
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
              <div>-</div>
            </div>
            <div>
              <div className="text-sm font-medium text-muted-foreground">시작일</div>
              <div>-</div>
            </div>
            <div>
              <div className="text-sm font-medium text-muted-foreground">완료일</div>
              <div>-</div>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>프롬프트</CardTitle>
          <CardDescription>태스크 실행에 사용되는 프롬프트</CardDescription>
        </CardHeader>
        <CardContent>
          <pre className="text-sm bg-muted p-4 rounded-lg">프롬프트 내용</pre>
        </CardContent>
      </Card>
    </div>
  )
}
