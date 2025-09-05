import './globals.css'
import { ReactNode } from 'react'

export const metadata = {
  title: 'ICP Payments Dashboard',
  description: 'Non-custodial payments dashboard',
}

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="ja" suppressHydrationWarning>
      <body className="min-h-screen bg-background text-foreground">
        <div className="mx-auto max-w-5xl p-6 space-y-6">
          <header className="flex items-center justify-between">
            <h1 className="text-xl font-semibold">ICP Payments Dashboard</h1>
          </header>
          {children}
        </div>
      </body>
    </html>
  )
}

