#!/usr/bin/env node
// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).

/**
 * Demiurge CLI - Official command-line interface
 * 
 * Complete toolkit for interacting with the Demiurge-Cloud:
 * - Blockchain queries and transactions
 * - QOR ID management
 * - DRC-369 NFT operations
 * - AI agent deployment
 * - Validator operations
 * 
 * DUAL MODE:
 * - Run `demiurge` with no arguments to launch interactive shell
 * - Run `demiurge <command>` for traditional CLI mode
 */

import { Command } from 'commander';
import chalk from 'chalk';
import { registerChainCommands } from './commands/chain';
import { registerWalletCommands } from './commands/wallet';
import { registerNftCommands } from './commands/nft';
import { registerAgentCommands } from './commands/agent';
import { registerIdentityCommands } from './commands/identity';
import { registerValidatorCommands } from './commands/validator';
import { registerDevCommands } from './commands/dev';

/**
 * Main entry point
 */
async function main(): Promise<void> {
  // Check if running in interactive mode (no commands)
  const hasArgs = process.argv.slice(2).filter(arg => !arg.startsWith('-')).length > 0;
  const forceInteractive = process.argv.includes('--interactive') || process.argv.includes('-i');
  const forceCommand = process.argv.includes('--cmd');
  
  if ((!hasArgs || forceInteractive) && !forceCommand) {
    // Launch interactive shell
    try {
      const { launchInteractiveShell } = await import('./shell');
      await launchInteractiveShell();
    } catch (error) {
      // Fallback to help if shell fails to load
      console.error(chalk.red('Failed to launch interactive shell:'), (error as Error).message);
      console.log(chalk.gray('Falling back to help...'));
      runCommandMode(['--help']);
    }
  } else {
    // Traditional command mode
    runCommandMode(process.argv);
  }
}

/**
 * Run in traditional command mode
 */
function runCommandMode(argv: string[]): void {
  const program = new Command();

  // CLI metadata
  program
    .name('demiurge')
    .description(chalk.cyan('Demiurge-Cloud CLI - The Sovereign Creative Substrate'))
    .version('1.0.0');

  // Register command groups
  registerChainCommands(program);
  registerWalletCommands(program);
  registerNftCommands(program);
  registerAgentCommands(program);
  registerIdentityCommands(program);
  registerValidatorCommands(program);
  registerDevCommands(program);

  // Global options
  program
    .option('-r, --rpc <url>', 'RPC endpoint URL', process.env.DEMIURGE_RPC_URL || 'http://127.0.0.1:9944')
    .option('-q, --quiet', 'Suppress output')
    .option('-j, --json', 'Output as JSON')
    .option('--no-color', 'Disable colored output')
    .option('-i, --interactive', 'Force interactive mode')
    .option('--cmd', 'Force command mode (for scripting)');

  // Add shell command for explicitly launching interactive mode
  program
    .command('shell')
    .description('Launch interactive shell')
    .action(async () => {
      const { launchInteractiveShell } = await import('./shell');
      await launchInteractiveShell();
    });

  // Parse and execute
  program.parse(argv);

  // If no command specified and not interactive, show enhanced help
  const args = argv.slice(2).filter(arg => !arg.startsWith('-'));
  if (args.length === 0) {
    console.log();
    console.log(chalk.cyan.bold('  DEMIURGE CLI'));
    console.log(chalk.gray('  Next-Level Blockchain'));
    console.log();
    console.log(chalk.white('  Usage:'));
    console.log(chalk.gray('    demiurge                  ') + chalk.cyan('Launch interactive shell'));
    console.log(chalk.gray('    demiurge <command>        ') + chalk.cyan('Run a command'));
    console.log(chalk.gray('    demiurge shell            ') + chalk.cyan('Launch interactive shell'));
    console.log();
    console.log(chalk.white('  Commands:'));
    console.log(chalk.gray('    wallet    ') + 'Manage wallets, keys, and CGT tokens');
    console.log(chalk.gray('    identity  ') + 'QOR ID and on-chain identity');
    console.log(chalk.gray('    nft       ') + 'DRC-369 NFT operations');
    console.log(chalk.gray('    agent     ') + 'Deploy and manage AI agents');
    console.log(chalk.gray('    chain     ') + 'Blockchain queries');
    console.log(chalk.gray('    validator ') + 'Validator operations');
    console.log(chalk.gray('    dev       ') + 'Development utilities');
    console.log();
    console.log(chalk.gray('  Run ') + chalk.white('demiurge <command> --help') + chalk.gray(' for more information.'));
    console.log();
  }
}

// Run main
main().catch((error) => {
  console.error(chalk.red('Fatal error:'), error.message);
  process.exit(1);
});
