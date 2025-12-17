import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'GameVault',
  description: 'Your personal game library',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-gv-dark">{children}</body>
    </html>
  )
}
