import { createRootRoute, Link, Outlet } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools'

export const Route = createRootRoute({
  component: () => (
    <div className="min-h-screen bg-background">
      <nav className="border-b">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center gap-6">
            <Link to="/" className="text-xl font-bold">
              AutoDev Dashboard
            </Link>
            <div className="flex gap-4">
              <Link
                to="/"
                className="text-foreground/60 hover:text-foreground transition-colors"
                activeProps={{
                  className: 'text-foreground font-medium',
                }}
              >
                대시보드
              </Link>
              <Link
                to="/tasks"
                className="text-foreground/60 hover:text-foreground transition-colors"
                activeProps={{
                  className: 'text-foreground font-medium',
                }}
              >
                태스크 목록
              </Link>
            </div>
          </div>
        </div>
      </nav>
      <main className="container mx-auto px-4 py-8">
        <Outlet />
      </main>
      <TanStackRouterDevtools />
    </div>
  ),
})
