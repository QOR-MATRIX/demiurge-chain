// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
#!/usr/bin/env tsx

import 'dotenv/config'
import * as readlineSync from 'readline-sync'
import { Sophia } from './sophia'

const SOPHIA_INTRO = `
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║         ╔═╗╔═╗╔═╗╦ ╦╔═╗╦ ╦  ╔═╗╔═╗╔═╗╦ ╦╔═╗╦ ╦            ║
║         ╚═╗║╣ ║ ║║║║║╣ ╠═╣  ╠═╝╠═╣║ ╦╠═╣║╣ ║║║            ║
║         ╚═╝╚═╝╚═╝╚╩╝╚═╝╩ ╩  ╩  ╩ ╩╚═╝╩ ╩╚═╝╚╩╝            ║
║                                                              ║
║         AI Assistant for Demiurge-Cloud                 ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝

Hello! I'm Sophia, your AI assistant for Demiurge-Cloud.

I can help you with:
  • Blockchain information and documentation
  • QOR ID authentication (login/signup)
  • Real-time chain service status
  • Development setup assistance
  • Troubleshooting chain services
  • Bug report submission

Type 'help' for commands, 'exit' to quit.
`

async function main() {
  console.log(SOPHIA_INTRO)
  
  if (!process.env.OPENAI_API_KEY) {
    console.error('❌ Error: OPENAI_API_KEY not found in environment variables')
    console.error('   Please set OPENAI_API_KEY in your .env file')
    process.exit(1)
  }

  const sophia = new Sophia({
    openaiApiKey: process.env.OPENAI_API_KEY!,
    qorAuthUrl: process.env.QOR_AUTH_URL || 'http://127.0.0.1:8080/api/v1',
    rpcUrl: process.env.RPC_URL || 'ws://127.0.0.1:9944',
    rpcHttpUrl: process.env.RPC_HTTP_URL || 'http://127.0.0.1:9944',
    bugReportEmail: process.env.BUG_REPORT_EMAIL || '',
  })

  console.log('\n✅ Sophia initialized and ready!\n')

  // Main chat loop
  while (true) {
    const input = readlineSync.question('You: ')
    
    if (!input || input.trim() === '') continue
    
    const command = input.trim().toLowerCase()
    
    // Handle special commands
    if (command === 'exit' || command === 'quit') {
      console.log('\n👋 Goodbye! Have a great day!')
      break
    }
    
    if (command === 'help') {
      console.log(`
Available commands:
  help              - Show this help message
  status            - Check chain status
  auth <action>     - QOR ID authentication (login/register)
  bug-report        - Submit a bug report
  exit / quit       - Exit Sophia

Or just ask me anything about Demiurge-Cloud!
`)
      continue
    }
    
    if (command === 'status') {
      await sophia.checkChainStatus()
      continue
    }
    
    if (command.startsWith('auth ')) {
      const action = command.split(' ')[1]
      if (action === 'login' || action === 'register') {
        await sophia.handleAuth(action)
      } else {
        console.log('Usage: auth <login|register>')
      }
      continue
    }
    
    if (command === 'bug-report') {
      await sophia.handleBugReport()
      continue
    }
    
    // Regular chat
    try {
      process.stdout.write('Sophia: ')
      await sophia.chat(input)
    } catch (error: any) {
      console.error(`\n❌ Error: ${error.message}`)
    }
  }
  
  process.exit(0)
}

main().catch((error) => {
  console.error('Fatal error:', error)
  process.exit(1)
})
