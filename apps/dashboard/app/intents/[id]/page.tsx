'use client'
import { useEffect, useState } from 'react'
import { useParams } from 'next/navigation'
import { Card, CardContent, CardHeader, CardTitle } from '../../../components/ui/card'
import { Button } from '../../../components/ui/button'
import { Input } from '../../../components/ui/input'

export default function IntentDetailPage() {
  const params = useParams<{ id: string }>()
  const id = decodeURIComponent(params.id)
  const base = process.env.NEXT_PUBLIC_BASE_URL || ''
  const [intent, setIntent] = useState<any>(null)
  const [owner, setOwner] = useState('')
  const [sub, setSub] = useState('')
  const [releaseTo, setReleaseTo] = useState('')
  const [releaseAmount, setReleaseAmount] = useState('')
  const [loading, setLoading] = useState(false)

  async function refresh() {
    const res = await fetch(`${base}/api/intents/${encodeURIComponent(id)}`, { cache: 'no-store' })
    const j = await res.json()
    setIntent('Some' in j ? j.Some : null)
  }
  useEffect(() => { refresh() }, [])

  async function doCapture() {
    setLoading(true)
    try {
      await fetch(`${base}/api/intents/${encodeURIComponent(id)}/capture`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ owner, subaccount: sub })
      })
      await refresh()
    } finally { setLoading(false) }
  }

  async function doRelease() {
    setLoading(true)
    try {
      await fetch(`${base}/api/intents/${encodeURIComponent(id)}/release`, {
        method: 'POST', headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ splits: [{ to: releaseTo, amount: releaseAmount }] })
      })
      await refresh()
    } finally { setLoading(false) }
  }

  async function doRefund() {
    setLoading(true)
    try {
      await fetch(`${base}/api/intents/${encodeURIComponent(id)}/refund`, {
        method: 'POST', headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ amount: intent?.amount })
      })
      await refresh()
    } finally { setLoading(false) }
  }

  if (!intent) return <p className="text-sm text-muted-foreground">Loading...</p>
  const status = Object.keys(intent.status)[0]
  return (
    <Card>
      <CardHeader>
        <CardTitle>Intent Detail</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        <div>
          <p className="font-mono text-xs">{id}</p>
          <p className="text-sm">asset: {intent.asset}</p>
          <p className="text-sm">amount: {intent.amount?.toString?.() ?? String(intent.amount)}</p>
          <p className="text-sm">status: {status}</p>
        </div>
        {status === 'RequiresApproval' && (
          <div className="space-y-2">
            <p className="font-medium">Capture</p>
            <div className="grid gap-2 md:grid-cols-3">
              <Input placeholder="payer owner (principal)" value={owner} onChange={(e) => setOwner(e.target.value)} />
              <Input placeholder="subaccount hex (optional)" value={sub} onChange={(e) => setSub(e.target.value)} />
              <Button onClick={doCapture} disabled={loading || !owner}>Capture</Button>
            </div>
          </div>
        )}
        {status === 'Succeeded' && (
          <div className="space-y-2">
            <p className="font-medium">Release / Refund</p>
            <div className="grid gap-2 md:grid-cols-3">
              <Input placeholder="to owner (principal)" value={releaseTo} onChange={(e) => setReleaseTo(e.target.value)} />
              <Input placeholder="amount (nat)" value={releaseAmount} onChange={(e) => setReleaseAmount(e.target.value)} />
              <div className="flex gap-2">
                <Button onClick={doRelease} disabled={loading || !releaseTo || !releaseAmount}>Release</Button>
                <Button variant="outline" onClick={doRefund} disabled={loading}>Refund</Button>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

