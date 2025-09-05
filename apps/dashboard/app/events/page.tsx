import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/card'

async function fetchEvents() {
  const base = process.env.NEXT_PUBLIC_BASE_URL || ''
  const res = await fetch(`${base}/api/events?offset=0&limit=50`, { cache: 'no-store' })
  return res.json()
}

export default async function EventsPage() {
  const data = await fetchEvents()
  const events = (data?.events ?? []) as any[]
  return (
    <Card>
      <CardHeader>
        <CardTitle>Events</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {events.map((e, i) => (
            <div key={i} className="text-sm border-b pb-2">
              <pre className="whitespace-pre-wrap break-all">{JSON.stringify(e, null, 2)}</pre>
            </div>
          ))}
          {events.length === 0 && <p className="text-sm text-muted-foreground">イベントがありません</p>}
        </div>
      </CardContent>
    </Card>
  )
}

