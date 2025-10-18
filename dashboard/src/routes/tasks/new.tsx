import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { apiClient } from '@/lib/api/client'

export const Route = createFileRoute('/tasks/new')({
  component: NewTask,
})

type TaskType = 'simple' | 'composite'

function NewTask() {
  const navigate = useNavigate()
  const [taskType, setTaskType] = useState<TaskType>('simple')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // 폼 상태
  const [repositoryOwner, setRepositoryOwner] = useState('')
  const [repositoryName, setRepositoryName] = useState('')
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [prompt, setPrompt] = useState('')
  const [compositePrompt, setCompositePrompt] = useState('')
  const [autoApprove, setAutoApprove] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsSubmitting(true)
    setError(null)

    try {
      if (taskType === 'simple') {
        const task = await apiClient.createTask({
          repository_owner: repositoryOwner,
          repository_name: repositoryName,
          title,
          description,
          prompt,
        })

        // 성공 시 해당 태스크 상세 페이지로 이동
        navigate({ to: `/tasks/${task.id}` })
      } else {
        const compositeTask = await apiClient.createCompositeTask({
          repository_owner: repositoryOwner,
          repository_name: repositoryName,
          title,
          description,
          composite_prompt: compositePrompt,
          auto_approve: autoApprove,
        })

        // 성공 시 해당 복합 태스크 상세 페이지로 이동
        navigate({ to: `/tasks/${compositeTask.id}` })
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '태스크 생성에 실패했습니다')
    } finally {
      setIsSubmitting(false)
    }
  }

  const handleCancel = () => {
    navigate({ to: '/tasks' })
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">새 태스크 생성</h1>
        <p className="text-muted-foreground">AutoDev 태스크를 생성합니다</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>태스크 정보</CardTitle>
          <CardDescription>모든 필드를 입력해주세요</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* 태스크 타입 선택 */}
            <div className="space-y-2">
              <Label>태스크 타입</Label>
              <div className="flex gap-4">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="taskType"
                    value="simple"
                    checked={taskType === 'simple'}
                    onChange={(e) => setTaskType(e.target.value as TaskType)}
                    className="w-4 h-4"
                  />
                  <span>단순 태스크 (Simple Task)</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="taskType"
                    value="composite"
                    checked={taskType === 'composite'}
                    onChange={(e) => setTaskType(e.target.value as TaskType)}
                    className="w-4 h-4"
                  />
                  <span>복합 태스크 (Composite Task)</span>
                </label>
              </div>
            </div>

            {/* 공통 필드 */}
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="repositoryOwner">Repository Owner *</Label>
                <Input
                  id="repositoryOwner"
                  value={repositoryOwner}
                  onChange={(e) => setRepositoryOwner(e.target.value)}
                  placeholder="예: myorg"
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="repositoryName">Repository Name *</Label>
                <Input
                  id="repositoryName"
                  value={repositoryName}
                  onChange={(e) => setRepositoryName(e.target.value)}
                  placeholder="예: myproject"
                  required
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="title">Title *</Label>
              <Input
                id="title"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="태스크 제목을 입력하세요"
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="description">Description *</Label>
              <Textarea
                id="description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="태스크에 대한 설명을 입력하세요"
                rows={3}
                required
              />
            </div>

            {/* 단순 태스크 전용 필드 */}
            {taskType === 'simple' && (
              <div className="space-y-2">
                <Label htmlFor="prompt">Prompt *</Label>
                <Textarea
                  id="prompt"
                  value={prompt}
                  onChange={(e) => setPrompt(e.target.value)}
                  placeholder="AI 에이전트에게 전달할 프롬프트를 입력하세요&#10;예: Add JWT authentication to the API. Include login and logout endpoints with proper error handling."
                  rows={6}
                  required
                />
              </div>
            )}

            {/* 복합 태스크 전용 필드 */}
            {taskType === 'composite' && (
              <>
                <div className="space-y-2">
                  <Label htmlFor="compositePrompt">Composite Prompt *</Label>
                  <Textarea
                    id="compositePrompt"
                    value={compositePrompt}
                    onChange={(e) => setCompositePrompt(e.target.value)}
                    placeholder="복합 태스크 프롬프트를 입력하세요. AI가 이를 여러 하위 태스크로 분해합니다.&#10;예: Review all RPC methods and fix security issues. Create one task per RPC method."
                    rows={6}
                    required
                  />
                </div>

                <div className="flex items-center justify-between rounded-lg border p-4">
                  <div className="space-y-0.5">
                    <Label htmlFor="autoApprove">Auto Approve</Label>
                    <div className="text-sm text-muted-foreground">
                      각 배치 실행 후 자동으로 다음 배치를 승인합니다
                    </div>
                  </div>
                  <Switch
                    id="autoApprove"
                    checked={autoApprove}
                    onCheckedChange={setAutoApprove}
                  />
                </div>
              </>
            )}

            {/* 에러 메시지 */}
            {error && (
              <div className="rounded-lg bg-destructive/15 p-4 text-sm text-destructive">
                {error}
              </div>
            )}

            {/* 버튼 */}
            <div className="flex gap-4">
              <Button
                type="submit"
                disabled={isSubmitting}
                className="flex-1"
              >
                {isSubmitting ? '생성 중...' : '태스크 생성'}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={handleCancel}
                disabled={isSubmitting}
                className="flex-1"
              >
                취소
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
