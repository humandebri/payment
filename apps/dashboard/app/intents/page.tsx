import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/card'
import { Button } from '../../components/ui/button'
import Link from 'next/link'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../../components/ui/table'

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
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>ID</TableHead>
              <TableHead>Asset</TableHead>
              <TableHead>Amount</TableHead>
              <TableHead>Status</TableHead>
              <TableHead></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {intents.map((it) => (
              <TableRow key={it.id}>
                <TableCell className="font-mono text-xs max-w-[280px] truncate" title={it.id}>{it.id}</TableCell>
                <TableCell>{it.asset}</TableCell>
                <TableCell>{it.amount.toString?.() ?? String(it.amount)}</TableCell>
                <TableCell>{Object.keys(it.status)[0]}</TableCell>
                <TableCell className="text-right">
                  <Link href={`/intents/${encodeURIComponent(it.id)}`}><Button size="sm" variant="outline">詳細</Button></Link>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
        {intents.length === 0 && <p className="text-sm text-muted-foreground">Intent がありません</p>}
      </CardContent>
    </Card>
  )
}
