// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
import Link from 'next/link'
import { Github, Twitter, Mail } from 'lucide-react'

export function Footer() {
  return (
    <footer className="glass-panel border-t border-neon-cyan/20 mt-auto">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          {/* Brand */}
          <div className="col-span-1 md:col-span-2">
            <h3 className="text-xl font-orbitron font-bold neon-text mb-4">Demiurge-Cloud</h3>
            <p className="text-gray-400 text-sm mb-4">
              The Web-First Metaverse Operating System. Build games, create NFTs, and mine CGT.
            </p>
            <div className="flex space-x-4">
              <a href="https://github.com/ALaustrup/demiurge-cloud" target="_blank" rel="noopener noreferrer" className="text-gray-400 hover:text-neon-cyan transition-colors">
                <Github className="w-5 h-5" />
              </a>
              <a href="https://twitter.com/demiurgechain" target="_blank" rel="noopener noreferrer" className="text-gray-400 hover:text-neon-cyan transition-colors">
                <Twitter className="w-5 h-5" />
              </a>
              <a href="https://github.com/QOR-MATRIX/QOR-OS/issues" className="text-gray-400 hover:text-neon-cyan transition-colors">
                <Mail className="w-5 h-5" />
              </a>
            </div>
          </div>

          {/* Quick Links */}
          <div>
            <h4 className="text-sm font-orbitron font-bold text-neon-cyan mb-4 uppercase">Resources</h4>
            <ul className="space-y-2 text-sm">
              <li><Link href="/docs" className="text-gray-400 hover:text-neon-cyan transition-colors">Documentation</Link></li>
              <li><Link href="/docs/getting-started" className="text-gray-400 hover:text-neon-cyan transition-colors">Getting Started</Link></li>
              <li><Link href="/cgt" className="text-gray-400 hover:text-neon-cyan transition-colors">CGT Mining</Link></li>
              <li><Link href="/blog" className="text-gray-400 hover:text-neon-cyan transition-colors">Chain News</Link></li>
            </ul>
          </div>

          {/* Community */}
          <div>
            <h4 className="text-sm font-orbitron font-bold text-neon-cyan mb-4 uppercase">Community</h4>
            <ul className="space-y-2 text-sm">
              <li><Link href="/forum" className="text-gray-400 hover:text-neon-cyan transition-colors">Forum</Link></li>
              <li><Link href="/blog" className="text-gray-400 hover:text-neon-cyan transition-colors">Blog</Link></li>
              <li><a href="https://github.com/ALaustrup/demiurge-cloud" target="_blank" rel="noopener noreferrer" className="text-gray-400 hover:text-neon-cyan transition-colors">GitHub</a></li>
            </ul>
          </div>
        </div>

        <div className="mt-8 pt-8 border-t border-neon-cyan/10 text-center text-sm text-gray-500">
          <p>&copy; {new Date().getFullYear()} Demiurge-Cloud. All rights reserved.</p>
          <p className="mt-2">Built with ❤️ by Alaustrup</p>
        </div>
      </div>
    </footer>
  )
}
