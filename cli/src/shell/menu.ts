// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Interactive menu system for Demiurge CLI
 */

import chalk from 'chalk';
import inquirer from 'inquirer';
import { showHeader, showBox } from './splash';
import { session, hasWallet, isAuthenticated, getSessionSummary } from './session';

/**
 * Menu option
 */
export interface MenuOption {
  key: string;
  label: string;
  description?: string;
  action: () => Promise<void | 'exit' | 'back'>;
  requiresWallet?: boolean;
  requiresAuth?: boolean;
}

/**
 * Menu definition
 */
export interface Menu {
  title: string;
  options: MenuOption[];
}

/**
 * Main menu structure
 */
export const MAIN_MENU: Menu = {
  title: 'Main Menu',
  options: [
    {
      key: '1',
      label: 'Identity & Keys',
      description: 'Generate, import, and manage keypairs',
      action: async () => { await showIdentityMenu(); return 'back'; },
    },
    {
      key: '2',
      label: 'Wallet',
      description: 'Balance, transfers, and transaction history',
      action: async () => { await showWalletMenu(); return 'back'; },
    },
    {
      key: '3',
      label: 'NFTs (DRC-369)',
      description: 'View, mint, and manage dynamic NFTs',
      action: async () => { await showNftMenu(); return 'back'; },
    },
    {
      key: '4',
      label: 'Chain Info',
      description: 'Block height, validators, and network status',
      action: async () => { await showChainInfo(); return 'back'; },
    },
    {
      key: '5',
      label: 'AI Agents',
      description: 'Deploy and manage autonomous agents',
      action: async () => { await showAgentMenu(); return 'back'; },
    },
    {
      key: '6',
      label: 'Settings',
      description: 'Configure endpoints and preferences',
      action: async () => { await showSettingsMenu(); return 'back'; },
    },
    {
      key: '7',
      label: 'Help',
      description: 'View help and documentation',
      action: async () => { await showHelp(); return 'back'; },
    },
    {
      key: 'q',
      label: 'Exit',
      description: 'Exit the CLI',
      action: async () => 'exit',
    },
  ],
};

/**
 * Display a menu and handle selection
 */
export async function showMenu(menu: Menu): Promise<'exit' | 'back' | void> {
  console.clear();
  showHeader();
  
  // Show session info
  const sessionInfo = getSessionSummary();
  if (sessionInfo.qorId || sessionInfo.wallet) {
    console.log(
      chalk.gray('  ') +
      (sessionInfo.qorId ? chalk.green(sessionInfo.qorId) : '') +
      (sessionInfo.wallet ? chalk.gray(' | ') + chalk.yellow(sessionInfo.wallet.slice(0, 10) + '...') : '')
    );
    console.log();
  }
  
  console.log(chalk.cyan.bold(`  ${menu.title}`));
  console.log();
  
  // Display options
  for (const option of menu.options) {
    const keyDisplay = chalk.cyan(`[${option.key}]`);
    const labelDisplay = option.label;
    const descDisplay = option.description ? chalk.gray(` - ${option.description}`) : '';
    
    // Check requirements
    let disabled = false;
    if (option.requiresWallet && !hasWallet()) disabled = true;
    if (option.requiresAuth && !isAuthenticated()) disabled = true;
    
    if (disabled) {
      console.log(chalk.gray(`  ${keyDisplay} ${labelDisplay}${descDisplay} [requires ${option.requiresWallet ? 'wallet' : 'auth'}]`));
    } else {
      console.log(`  ${keyDisplay} ${labelDisplay}${descDisplay}`);
    }
  }
  
  console.log();
  
  // Get selection
  const { choice } = await inquirer.prompt([
    {
      type: 'input',
      name: 'choice',
      message: chalk.gray('Select option:'),
      prefix: '',
    },
  ]);
  
  const selected = menu.options.find(o => o.key.toLowerCase() === choice.toLowerCase());
  
  if (!selected) {
    console.log(chalk.red('Invalid option. Please try again.'));
    await sleep(1000);
    return showMenu(menu);
  }
  
  // Check requirements
  if (selected.requiresWallet && !hasWallet()) {
    console.log(chalk.red('This option requires a wallet. Please generate or import one first.'));
    await sleep(1500);
    return showMenu(menu);
  }
  
  if (selected.requiresAuth && !isAuthenticated()) {
    console.log(chalk.red('This option requires authentication. Please login first.'));
    await sleep(1500);
    return showMenu(menu);
  }
  
  return selected.action();
}

/**
 * Run the main menu loop
 */
export async function runMainMenu(): Promise<void> {
  while (true) {
    const result = await showMenu(MAIN_MENU);
    if (result === 'exit') {
      console.log(chalk.cyan('\n  Goodbye!\n'));
      process.exit(0);
    }
  }
}

// =============================================================================
// Submenu Implementations
// =============================================================================

async function showIdentityMenu(): Promise<void> {
  const menu: Menu = {
    title: 'Identity & Keys',
    options: [
      {
        key: '1',
        label: 'Generate New Keypair',
        action: async () => {
          console.log(chalk.cyan('\n  Generating new keypair...\n'));
          // TODO: Implement with SDK
          const mockPubkey = '0x' + Array(64).fill(0).map(() => Math.floor(Math.random() * 16).toString(16)).join('');
          console.log(chalk.green('  Public Key: ') + chalk.white(mockPubkey));
          console.log(chalk.yellow('\n  Save your private key securely!'));
          await waitForKey();
        },
      },
      {
        key: '2',
        label: 'Import Existing Key',
        action: async () => {
          const { privateKey } = await inquirer.prompt([
            {
              type: 'password',
              name: 'privateKey',
              message: 'Enter private key:',
              mask: '*',
            },
          ]);
          console.log(chalk.green('\n  Key imported successfully!'));
          await waitForKey();
        },
      },
      {
        key: '3',
        label: 'Link to QOR ID',
        action: async () => {
          console.log(chalk.gray('\n  QOR ID linking requires authentication.'));
          console.log(chalk.gray('  Visit: http://127.0.0.1:8080\n'));
          await waitForKey();
        },
      },
      {
        key: '4',
        label: 'View Current Identity',
        action: async () => {
          const info = getSessionSummary();
          if (info.qorId) {
            console.log(chalk.cyan('\n  QOR ID: ') + chalk.white(info.qorId));
          }
          if (info.wallet) {
            console.log(chalk.cyan('  Wallet: ') + chalk.white(info.wallet));
          }
          if (!info.qorId && !info.wallet) {
            console.log(chalk.gray('\n  No identity loaded. Generate or import a key first.'));
          }
          await waitForKey();
        },
      },
      {
        key: 'b',
        label: 'Back',
        action: async () => 'back',
      },
    ],
  };
  
  await showMenu(menu);
}

async function showWalletMenu(): Promise<void> {
  const menu: Menu = {
    title: 'Wallet',
    options: [
      {
        key: '1',
        label: 'Check Balance',
        action: async () => {
          console.log(chalk.cyan('\n  Fetching balance...\n'));
          console.log(chalk.green('  Balance: ') + chalk.white.bold('0.00') + chalk.gray(' CGT'));
          console.log(chalk.gray('  Sparks: 0'));
          await waitForKey();
        },
      },
      {
        key: '2',
        label: 'Send CGT',
        requiresWallet: true,
        action: async () => {
          const answers = await inquirer.prompt([
            { type: 'input', name: 'to', message: 'Recipient address:' },
            { type: 'input', name: 'amount', message: 'Amount (CGT):' },
          ]);
          console.log(chalk.yellow(`\n  Sending ${answers.amount} CGT to ${answers.to.slice(0, 10)}...`));
          console.log(chalk.gray('  [Transaction simulation - not actually sending]'));
          await waitForKey();
        },
      },
      {
        key: '3',
        label: 'Transaction History',
        action: async () => {
          console.log(chalk.gray('\n  No transactions found.'));
          await waitForKey();
        },
      },
      {
        key: '4',
        label: 'Energy Level',
        action: async () => {
          console.log(chalk.cyan('\n  Energy: ') + chalk.green('100') + chalk.gray('/100'));
          const bar = chalk.green('█'.repeat(20));
          console.log('  ' + bar + ' 100%');
          await waitForKey();
        },
      },
      {
        key: 'b',
        label: 'Back',
        action: async () => 'back',
      },
    ],
  };
  
  await showMenu(menu);
}

async function showNftMenu(): Promise<void> {
  const menu: Menu = {
    title: 'NFTs (DRC-369)',
    options: [
      { key: '1', label: 'View My NFTs', action: async () => { console.log(chalk.gray('\n  No NFTs found.')); await waitForKey(); } },
      { key: '2', label: 'Mint New NFT', requiresWallet: true, action: async () => { console.log(chalk.gray('\n  Minting not yet implemented.')); await waitForKey(); } },
      { key: '3', label: 'Transfer NFT', requiresWallet: true, action: async () => { console.log(chalk.gray('\n  Transfer not yet implemented.')); await waitForKey(); } },
      { key: 'b', label: 'Back', action: async () => 'back' },
    ],
  };
  await showMenu(menu);
}

async function showChainInfo(): Promise<void> {
  console.log(chalk.cyan('\n  Chain Status\n'));
  console.log(chalk.white('  Block Height: ') + chalk.green('0'));
  console.log(chalk.white('  Chain ID:     ') + chalk.gray('demiurge-mainnet-v1'));
  console.log(chalk.white('  Validators:   ') + chalk.yellow('4'));
  console.log(chalk.white('  TPS:          ') + chalk.gray('0'));
  console.log(chalk.white('  RPC:          ') + chalk.gray(session.rpcEndpoint));
  await waitForKey();
}

async function showAgentMenu(): Promise<void> {
  const menu: Menu = {
    title: 'AI Agents',
    options: [
      {
        key: '1',
        label: 'Deploy New Agent',
        requiresAuth: true,
        action: async () => {
          console.log(chalk.gray('\n  Agent deployment wizard coming soon.'));
          await waitForKey();
        },
      },
      { key: '2', label: 'List My Agents', action: async () => { console.log(chalk.gray('\n  No agents deployed.')); await waitForKey(); } },
      { key: '3', label: 'Agent Status', action: async () => { console.log(chalk.gray('\n  Enter agent DID to check status.')); await waitForKey(); } },
      { key: '4', label: 'Register Agent (QOR ID)', requiresAuth: true, action: async () => { console.log(chalk.gray('\n  Agent registration coming soon.')); await waitForKey(); } },
      { key: 'b', label: 'Back', action: async () => 'back' },
    ],
  };
  await showMenu(menu);
}

async function showSettingsMenu(): Promise<void> {
  const menu: Menu = {
    title: 'Settings',
    options: [
      {
        key: '1',
        label: 'Change RPC Endpoint',
        action: async () => {
          const { endpoint } = await inquirer.prompt([
            { type: 'input', name: 'endpoint', message: 'RPC Endpoint:', default: session.rpcEndpoint },
          ]);
          session.rpcEndpoint = endpoint;
          console.log(chalk.green('\n  Endpoint updated.'));
          await waitForKey();
        },
      },
      {
        key: '2',
        label: 'Toggle Animations',
        action: async () => {
          console.log(chalk.gray('\n  Animation settings toggled.'));
          await waitForKey();
        },
      },
      { key: 'b', label: 'Back', action: async () => 'back' },
    ],
  };
  await showMenu(menu);
}

async function showHelp(): Promise<void> {
  showBox('Demiurge CLI Help', [
    chalk.white('Commands:'),
    chalk.gray('  help      - Show this help'),
    chalk.gray('  wallet    - Wallet operations'),
    chalk.gray('  nft       - NFT operations'),
    chalk.gray('  chain     - Chain info'),
    chalk.gray('  agent     - AI agents'),
    chalk.gray('  exit      - Exit CLI'),
    '',
    chalk.white('Keyboard:'),
    chalk.gray('  1-9       - Select menu option'),
    chalk.gray('  b         - Go back'),
    chalk.gray('  q         - Exit'),
    '',
    chalk.white('Documentation:'),
    chalk.cyan('  https://qorsync.dev'),
  ]);
  await waitForKey();
}

// =============================================================================
// Helpers
// =============================================================================

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function waitForKey(): Promise<void> {
  console.log(chalk.gray('\n  Press Enter to continue...'));
  await inquirer.prompt([{ type: 'input', name: 'continue', message: '', prefix: '' }]);
}
