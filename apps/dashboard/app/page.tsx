import Link from 'next/link'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { Button } from '../components/ui/button'

export default function Page() {
  return (
    <div className="grid gap-6 md:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Intents</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground mb-4">Intent の作成・一覧・操作</p>
          <div className="flex gap-3">
            <Link href="/intents"><Button>一覧を見る</Button></Link>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Events</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground mb-4">イベントログ（認証付き）</p>
          <div className="flex gap-3">
            <Link href="/events"><Button variant="outline">イベントを見る</Button></Link>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

