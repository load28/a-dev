const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000'

export interface Task {
  id: string
  title: string
  description: string
  prompt: string
  task_type: 'Simple' | 'Composite'
  status: 'Pending' | 'WaitingDependencies' | 'Ready' | 'InProgress' | 'Completed' | 'Failed' | 'Cancelled'
  dependencies: string[]
  created_at: string
  started_at: string | null
  completed_at: string | null
  pr_url: string | null
  workflow_run_id: string | null
  error: string | null
  auto_approve: boolean
}

export interface CreateTaskRequest {
  repository_owner: string
  repository_name: string
  title: string
  description: string
  prompt: string
}

export interface Stats {
  total_tasks: number
  pending_tasks: number
  in_progress_tasks: number
  completed_tasks: number
  failed_tasks: number
}

class ApiClient {
  private baseUrl: string

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl
  }

  private async request<T>(path: string, options?: RequestInit): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    })

    if (!response.ok) {
      throw new Error(`API Error: ${response.statusText}`)
    }

    return response.json()
  }

  // Task endpoints
  async getTasks(): Promise<Task[]> {
    return this.request<Task[]>('/tasks')
  }

  async getTask(taskId: string): Promise<Task> {
    return this.request<Task>(`/tasks/${taskId}`)
  }

  async createTask(data: CreateTaskRequest): Promise<Task> {
    return this.request<Task>('/tasks', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async executeTask(taskId: string): Promise<void> {
    return this.request<void>(`/tasks/${taskId}/execute`, {
      method: 'POST',
    })
  }

  // Stats endpoint
  async getStats(): Promise<Stats> {
    return this.request<Stats>('/stats')
  }

  // Health check
  async healthCheck(): Promise<{ status: string }> {
    return this.request<{ status: string }>('/health')
  }
}

export const apiClient = new ApiClient(API_URL)
