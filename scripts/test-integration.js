#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Set up environment variables with defaults
process.env.ANCHOR_PROVIDER_URL = process.env.ANCHOR_PROVIDER_URL || 'http://127.0.0.1:8899';
process.env.ANCHOR_WALLET = process.env.ANCHOR_WALLET || path.join(require('os').homedir(), '.config/solana/id.json');

// Change to the anchor project directory
const anchorDir = path.join(__dirname, '../contracts/solana/universal_nft');
process.chdir(anchorDir);

// Ensure shared directory exists in test directory and copy shared files
const testSharedDir = path.join(process.cwd(), '../../../test/integration/shared');
const rootSharedDir = path.join(__dirname, '../shared');

if (!fs.existsSync(testSharedDir)) {
  fs.mkdirSync(testSharedDir, { recursive: true });
}

// Copy shared TypeScript files to test directory
const sharedFiles = ['CrossChainMessage.ts', 'MessageBridge.ts'];
sharedFiles.forEach(file => {
  const src = path.join(rootSharedDir, file);
  const dest = path.join(testSharedDir, file);
  if (fs.existsSync(src)) {
    fs.copyFileSync(src, dest);
    console.log(`📄 Copied ${file} to test/integration/shared`);
  }
});

console.log(`🔧 Running integration tests from: ${process.cwd()}`);
console.log(`📡 Anchor Provider URL: ${process.env.ANCHOR_PROVIDER_URL}`);
console.log(`👛 Anchor Wallet: ${process.env.ANCHOR_WALLET}`);

// Run the test command
const testCommand = [
  'mocha',
  '--require', 'ts-node/register',
  '../../../test/integration/**/*.test.ts',
  '--timeout', '60000',
  '--recursive'
];

const child = spawn('npx', testCommand, {
  stdio: 'inherit',
  env: process.env
});

child.on('close', (code) => {
  process.exit(code);
});

child.on('error', (err) => {
  console.error('Failed to start test process:', err);
  process.exit(1);
});