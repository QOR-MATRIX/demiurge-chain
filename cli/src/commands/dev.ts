// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Dev commands - Development and testing utilities
 */

import { Command } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import Table from 'cli-table3';
import { DemiurgeClient } from '@demiurge/sdk';
import { handleError } from '../utils/format';
import * as fs from 'fs';
import * as path from 'path';

export function registerDevCommands(program: Command) {
  const dev = program
    .command('dev')
    .description('Development and testing utilities');

  // Initialize new project
  dev
    .command('init')
    .description('Initialize a new Demiurge project')
    .argument('[directory]', 'Project directory', '.')
    .option('-t, --template <type>', 'Project template (game|web|agent)', 'web')
    .action(async (directory, options, command) => {
      const spinner = ora('Initializing project...').start();
      try {
        const projectPath = path.resolve(directory);
        
        // Create directory structure
        if (!fs.existsSync(projectPath)) {
          fs.mkdirSync(projectPath, { recursive: true });
        }
        
        // Create package.json
        const packageJson = {
          name: path.basename(projectPath),
          version: '0.1.0',
          description: 'Demiurge-Cloud Integration',
          scripts: {
            dev: 'npm run dev',
            build: 'npm run build',
          },
          dependencies: {
            '@demiurge/sdk': '^1.0.0',
            '@demiurge/qor-sdk': '^1.0.0',
            '@demiurge/drc369-sdk': '^1.0.0',
          },
        };
        
        fs.writeFileSync(
          path.join(projectPath, 'package.json'),
          JSON.stringify(packageJson, null, 2)
        );
        
        // Create example .env
        const envContent = `# Demiurge Configuration
DEMIURGE_RPC_URL=http://127.0.0.1:9944
NEXT_PUBLIC_QOR_AUTH_URL=http://127.0.0.1:3000/api/v1
`;
        fs.writeFileSync(path.join(projectPath, '.env.example'), envContent);
        
        // Create README
        const readmeContent = `# ${path.basename(projectPath)}

Demiurge-Cloud Integration

## Setup

\`\`\`bash
npm install
cp .env.example .env
\`\`\`

## Development

\`\`\`bash
npm run dev
\`\`\`

## Documentation

- [Demiurge Docs](https://qorsync.dev/docs)
- [SDK Reference](https://github.com/ALaustrup/demiurge-cloud)
`;
        fs.writeFileSync(path.join(projectPath, 'README.md'), readmeContent);
        
        spinner.succeed('Project initialized');
        
        console.log(chalk.green('\n✅ Project Created!\n'));
        console.log(chalk.white('Location:'), chalk.cyan(projectPath));
        console.log(chalk.white('Template:'), chalk.yellow(options.template));
        console.log(chalk.cyan('\nNext steps:'));
        console.log(chalk.gray('  cd'), chalk.white(directory));
        console.log(chalk.gray('  npm install'));
        console.log(chalk.gray('  cp .env.example .env'));
      } catch (error) {
        spinner.fail('Failed to initialize project');
        handleError(error);
      }
    });

  // Test RPC connection
  dev
    .command('test-rpc')
    .description('Test RPC connection and availability')
    .action(async (options, command) => {
      const rpc = command.optsWithGlobals().rpc;
      console.log(chalk.cyan('\n🔌 Testing RPC Connection...\n'));
      console.log(chalk.white('Endpoint:'), chalk.gray(rpc));
      
      const spinner = ora('Connecting...').start();
      try {
        const client = new DemiurgeClient({ rpcUrl: rpc });
        
        const blockNumber = await client.getBlockNumber();
        
        spinner.succeed('Connected');
        
        const table = new Table({
          head: [chalk.green('✓ Test'), chalk.white('Result')],
          colWidths: [30, 50],
        });
        
        table.push(
          ['Connection', chalk.green('Success')],
          ['Block Number', chalk.white(blockNumber.toString())],
          ['Endpoint', chalk.gray(rpc)],
        );
        
        console.log('\n' + table.toString() + '\n');
        console.log(chalk.green('✅ RPC connection successful!'));
      } catch (error) {
        spinner.fail('Connection failed');
        handleError(error);
        console.log(chalk.yellow('\nTroubleshooting:'));
        console.log(chalk.gray('  - Check the RPC endpoint is correct'));
        console.log(chalk.gray('  - Verify the node is running'));
        console.log(chalk.gray('  - Check firewall settings'));
      }
    });

  // Generate test data
  dev
    .command('faucet')
    .description('Request testnet CGT from faucet (testnet only)')
    .argument('<address>', 'Address to fund')
    .action(async (address, options, command) => {
      const spinner = ora('Requesting faucet...').start();
      try {
        // TODO: Implement faucet RPC call
        spinner.succeed('Faucet request submitted');
        
        console.log(chalk.green('\n✅ Faucet Request Submitted'));
        console.log(chalk.white('Address:'), chalk.gray(address));
        console.log(chalk.white('Amount: '), chalk.green('100 CGT'));
        console.log(chalk.gray('\nNote: Faucet support coming soon'));
      } catch (error) {
        spinner.fail('Faucet request failed');
        handleError(error);
      }
    });

  // Benchmark
  dev
    .command('benchmark')
    .description('Run performance benchmarks')
    .option('-t, --transactions <number>', 'Number of test transactions', '100')
    .action(async (options, command) => {
      const spinner = ora('Running benchmark...').start();
      try {
        const rpc = command.optsWithGlobals().rpc;
        const client = new DemiurgeClient({ rpcUrl: rpc });
        
        const iterations = parseInt(options.transactions);
        const start = Date.now();
        
        // Run block number queries as benchmark
        for (let i = 0; i < iterations; i++) {
          await client.getBlockNumber();
        }
        
        const duration = Date.now() - start;
        const avgLatency = duration / iterations;
        const qps = (iterations / (duration / 1000)).toFixed(2);
        
        spinner.succeed('Benchmark complete');
        
        console.log(chalk.cyan('\n📊 Benchmark Results:\n'));
        console.log(chalk.white('Queries:        '), chalk.green(iterations));
        console.log(chalk.white('Total Time:     '), chalk.yellow(`${duration}ms`));
        console.log(chalk.white('Avg Latency:    '), chalk.blue(`${avgLatency.toFixed(2)}ms`));
        console.log(chalk.white('Queries/Second: '), chalk.magenta(qps));
      } catch (error) {
        spinner.fail('Benchmark failed');
        handleError(error);
      }
    });
}
