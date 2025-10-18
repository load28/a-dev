import { Badge } from '@/components/ui/badge'
import { Loader2, Clock, Check, X, Circle, PlayCircle } from 'lucide-react'

interface TaskStatusBadgeProps {
  status: string
}

export function TaskStatusBadge({ status }: TaskStatusBadgeProps) {
  const config: Record<string, {
    variant: 'default' | 'secondary' | 'destructive' | 'outline'
    icon: React.ReactNode
    label: string
    className?: string
  }> = {
    Pending: {
      variant: 'secondary',
      icon: <Clock className="w-3 h-3" />,
      label: '대기 중',
    },
    WaitingDependencies: {
      variant: 'secondary',
      icon: <Clock className="w-3 h-3" />,
      label: '의존성 대기',
    },
    Ready: {
      variant: 'outline',
      icon: <Circle className="w-3 h-3" />,
      label: '준비됨',
    },
    InProgress: {
      variant: 'default',
      icon: <Loader2 className="w-3 h-3 animate-spin" />,
      label: '실행 중',
    },
    Completed: {
      variant: 'default',
      icon: <Check className="w-3 h-3" />,
      label: '완료',
      className: 'bg-green-600 hover:bg-green-700',
    },
    Failed: {
      variant: 'destructive',
      icon: <X className="w-3 h-3" />,
      label: '실패',
    },
    Cancelled: {
      variant: 'secondary',
      icon: <X className="w-3 h-3" />,
      label: '취소됨',
    },
  }

  const { variant, icon, label, className } = config[status] || config.Pending

  return (
    <Badge variant={variant} className={className}>
      <span className="flex items-center gap-1.5">
        {icon}
        <span>{label}</span>
      </span>
    </Badge>
  )
}
