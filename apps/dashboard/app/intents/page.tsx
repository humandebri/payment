import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/card'
import { Button } from '../../components/ui/button'

async function fetchIntents() {
  const base = process.env.NEXT_PUBLIC_BASE_URL || ''
  const res = await fetch(`${base}/api/intents`, { cache: 'no-store' })
  return res.json()
}

export default async function IntentsPage() {
  const data = await fetchIntents()
  const intents = (data?.intents ?? []) as any[]
  return (
    <Card>
      <CardHeader>
        <CardTitle>Intents</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {intents.map((it) => (
            <div key={it.id} className="border p-3 rounded-md">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-mono text-xs text-muted-foreground">{it.id}</p>
                  <p className="text-sm">{it.asset} / amount: {it.amount.toString?.() ?? String(it.amount)}</p>
                  <p className="text-sm">status: {Object.keys(it.status)[0]}</p>
                </div>
                <div className="flex gap-2">
                  <Button variant="outline" disabled>詳細</Button>
                </div>
              </div>
            </div>
          ))}
          {intents.length === 0 && <p className="text-sm text-muted-foreground">Intent がありません</p>}
        </div>
      </CardContent>
    </Card>
  )
}

