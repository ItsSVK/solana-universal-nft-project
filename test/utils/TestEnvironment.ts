import { ethers } from 'ethers';
import { Connection, Keypair, PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { expect } from 'chai';

/**
 * Test Environment Utility for Universal NFT Protocol
 * Provides standardized setup and utilities for end-to-end testing
 */

export interface TestConfig {
  // Network configurations
  ethRpcUrl?: string;
  solanaRpcUrl?: string;
  
  // Test timeouts
  defaultTimeout?: number;
  networkTimeout?: number;
  
  // Funding amounts
  ethFundingAmount?: string;
  solanaFundingAmount?: number;
  
  // Contract addresses (for deployed testing)
  zetaChainNFTAddress?: string;
  baseNFTReceiverAddress?: string;
  gatewayAddress?: string;
}

export interface TestWallets {
  // Ethereum wallets
  deployer: ethers.Wallet;
  alice: ethers.Wallet;
  bob: ethers.Wallet;
  charlie: ethers.Wallet;
  mockGateway: ethers.Wallet;
  
  // Solana wallets
  aliceSol: Keypair;
  bobSol: Keypair;
  charlieSol: Keypair;
}

export interface TestContracts {
  zetaChainNFT?: any;
  baseNFTReceiver?: any;
  mockGateway?: any;
}

export interface NetworkStatus {
  ethereum: {
    connected: boolean;
    blockNumber?: number;
    chainId?: number;
  };
  solana: {
    connected: boolean;
    version?: any;
    slot?: number;
  };
}

export class TestEnvironment {
  public config: TestConfig;
  public wallets: TestWallets;
  public ethProvider: ethers.JsonRpcProvider;
  public solanaConnection: Connection;
  public contracts: TestContracts = {};

  constructor(config: TestConfig = {}) {
    this.config = {
      ethRpcUrl: 'http://localhost:8545',
      solanaRpcUrl: 'http://localhost:8899',
      defaultTimeout: 30000,
      networkTimeout: 10000,
      ethFundingAmount: '10.0',
      solanaFundingAmount: 10,
      ...config
    };

    // Initialize providers
    this.ethProvider = new ethers.JsonRpcProvider(this.config.ethRpcUrl);
    this.solanaConnection = new Connection(this.config.solanaRpcUrl!, 'confirmed');

    // Initialize wallets
    this.wallets = this.createTestWallets();
  }

  /**
   * Create standardized test wallets
   */
  private createTestWallets(): TestWallets {
    return {
      // Ethereum wallets with deterministic keys for testing
      deployer: new ethers.Wallet('0x' + '00'.repeat(32), this.ethProvider),
      alice: new ethers.Wallet('0x' + '01'.repeat(32), this.ethProvider),
      bob: new ethers.Wallet('0x' + '02'.repeat(32), this.ethProvider),
      charlie: new ethers.Wallet('0x' + '03'.repeat(32), this.ethProvider),
      mockGateway: new ethers.Wallet('0x' + '99'.repeat(32), this.ethProvider),
      
      // Solana wallets
      aliceSol: Keypair.generate(),
      bobSol: Keypair.generate(),
      charlieSol: Keypair.generate(),
    };
  }

  /**
   * Setup the test environment
   */
  async setup(): Promise<void> {
    console.log('🔧 Setting up test environment...');
    
    // Check network connectivity
    const status = await this.checkNetworkStatus();
    
    if (!status.ethereum.connected) {
      throw new Error('Ethereum network not accessible');
    }
    
    if (!status.solana.connected) {
      throw new Error('Solana network not accessible'); 
    }

    console.log(`   ✅ Ethereum: Block ${status.ethereum.blockNumber} (Chain ${status.ethereum.chainId})`);
    console.log(`   ✅ Solana: Slot ${status.solana.slot} (Version ${status.solana.version?.['solana-core']})`);

    // Fund test wallets if needed
    await this.fundTestWallets();

    console.log('   ✅ Test environment ready!');
  }

  /**
   * Check network connectivity status
   */
  async checkNetworkStatus(): Promise<NetworkStatus> {
    const status: NetworkStatus = {
      ethereum: { connected: false },
      solana: { connected: false }
    };

    try {
      status.ethereum.blockNumber = await this.ethProvider.getBlockNumber();
      const network = await this.ethProvider.getNetwork();
      status.ethereum.chainId = Number(network.chainId);
      status.ethereum.connected = true;
    } catch (error) {
      console.warn('Ethereum network check failed:', (error as Error).message);
    }

    try {
      status.solana.version = await this.solanaConnection.getVersion();
      status.solana.slot = await this.solanaConnection.getSlot();
      status.solana.connected = true;
    } catch (error) {
      console.warn('Solana network check failed:', (error as Error).message);
    }

    return status;
  }

  /**
   * Fund test wallets with native tokens
   */
  async fundTestWallets(): Promise<void> {
    // Note: In real testing environments, you'd need actual funding mechanisms
    // For local testing, ensure your local networks have funded accounts
    
    console.log('💰 Checking wallet funding...');
    
    // Check ETH balances
    const ethWallets = [this.wallets.deployer, this.wallets.alice, this.wallets.bob, this.wallets.charlie];
    for (const wallet of ethWallets) {
      const balance = await this.ethProvider.getBalance(wallet.address);
      console.log(`   ETH ${wallet.address.slice(0, 8)}...: ${ethers.formatEther(balance)} ETH`);
    }

    // Check SOL balances  
    const solWallets = [this.wallets.aliceSol, this.wallets.bobSol, this.wallets.charlieSol];
    for (const wallet of solWallets) {
      try {
        const balance = await this.solanaConnection.getBalance(wallet.publicKey);
        console.log(`   SOL ${wallet.publicKey.toString().slice(0, 8)}...: ${balance / LAMPORTS_PER_SOL} SOL`);
      } catch (error) {
        console.log(`   SOL ${wallet.publicKey.toString().slice(0, 8)}...: 0 SOL (new wallet)`);
      }
    }
  }

  /**
   * Deploy test contracts
   */
  async deployContracts(): Promise<void> {
    console.log('📋 Deploying test contracts...');

    // Deploy ZetaChain Universal NFT
    const UniversalNFTFactory = await ethers.getContractFactory('UniversalNFT', this.wallets.deployer);
    this.contracts.zetaChainNFT = await UniversalNFTFactory.deploy(
      this.wallets.mockGateway.address,
      this.wallets.deployer.address
    );
    await this.contracts.zetaChainNFT.waitForDeployment();

    // Deploy Base Sepolia NFT Receiver
    const UniversalNFTReceiverFactory = await ethers.getContractFactory('UniversalNFTReceiver', this.wallets.deployer);
    this.contracts.baseNFTReceiver = await UniversalNFTReceiverFactory.deploy(
      this.wallets.mockGateway.address,
      this.wallets.deployer.address,
      await this.contracts.zetaChainNFT.getAddress()
    );
    await this.contracts.baseNFTReceiver.waitForDeployment();

    console.log(`   ✅ ZetaChain NFT: ${await this.contracts.zetaChainNFT.getAddress()}`);
    console.log(`   ✅ Base NFT Receiver: ${await this.contracts.baseNFTReceiver.getAddress()}`);
  }

  /**
   * Clean up test environment
   */
  async cleanup(): Promise<void> {
    console.log('🧹 Cleaning up test environment...');
    
    // Close connections if needed
    // In most cases, connections will close automatically
    
    console.log('   ✅ Cleanup complete');
  }

  /**
   * Create standardized test NFT metadata
   */
  createTestNFTMetadata(params: {
    tokenId: string;
    name: string;
    description: string;
    attributes?: Array<{ trait_type: string; value: string }>;
  }): string {
    const metadata = {
      name: params.name,
      description: params.description,
      image: `https://api.universalnft.com/images/test-${params.tokenId}.png`,
      attributes: params.attributes || [
        { trait_type: 'Test', value: 'true' },
        { trait_type: 'Token ID', value: params.tokenId }
      ]
    };

    return `data:application/json;base64,${Buffer.from(JSON.stringify(metadata)).toString('base64')}`;
  }

  /**
   * Wait for transaction confirmation with timeout
   */
  async waitForTransaction(
    txPromise: Promise<any>, 
    timeoutMs: number = this.config.networkTimeout!
  ): Promise<any> {
    return Promise.race([
      txPromise,
      new Promise((_, reject) => 
        setTimeout(() => reject(new Error('Transaction timeout')), timeoutMs)
      )
    ]);
  }

  /**
   * Verify contract deployment
   */
  async verifyContracts(): Promise<boolean> {
    try {
      if (this.contracts.zetaChainNFT) {
        const name = await this.contracts.zetaChainNFT.name();
        expect(name).to.equal('Universal NFT');
      }

      if (this.contracts.baseNFTReceiver) {
        const name = await this.contracts.baseNFTReceiver.name();
        expect(name).to.equal('Universal NFT Receiver');
      }

      return true;
    } catch (error) {
      console.error('Contract verification failed:', error);
      return false;
    }
  }

  /**
   * Get test configuration summary
   */
  getConfigSummary(): any {
    return {
      networks: {
        ethereum: this.config.ethRpcUrl,
        solana: this.config.solanaRpcUrl
      },
      timeouts: {
        default: this.config.defaultTimeout,
        network: this.config.networkTimeout
      },
      wallets: {
        ethereum: {
          deployer: this.wallets.deployer.address,
          alice: this.wallets.alice.address,
          bob: this.wallets.bob.address,
          charlie: this.wallets.charlie.address
        },
        solana: {
          alice: this.wallets.aliceSol.publicKey.toString(),
          bob: this.wallets.bobSol.publicKey.toString(),
          charlie: this.wallets.charlieSol.publicKey.toString()
        }
      },
      contracts: {
        zetaChainNFT: this.contracts.zetaChainNFT ? 'Deployed' : 'Not deployed',
        baseNFTReceiver: this.contracts.baseNFTReceiver ? 'Deployed' : 'Not deployed'
      }
    };
  }

  /**
   * Generate test report
   */
  generateTestReport(testResults: any): string {
    const report = `
# Universal NFT Protocol Test Report

## Environment Configuration
- Ethereum RPC: ${this.config.ethRpcUrl}
- Solana RPC: ${this.config.solanaRpcUrl}
- Default Timeout: ${this.config.defaultTimeout}ms

## Test Wallets
### Ethereum
- Deployer: ${this.wallets.deployer.address}
- Alice: ${this.wallets.alice.address}
- Bob: ${this.wallets.bob.address}
- Charlie: ${this.wallets.charlie.address}

### Solana  
- Alice: ${this.wallets.aliceSol.publicKey.toString()}
- Bob: ${this.wallets.bobSol.publicKey.toString()}
- Charlie: ${this.wallets.charlieSol.publicKey.toString()}

## Test Results
${JSON.stringify(testResults, null, 2)}

## Generated: ${new Date().toISOString()}
`;

    return report;
  }
}

/**
 * Helper function to create test environment
 */
export function createTestEnvironment(config: TestConfig = {}): TestEnvironment {
  return new TestEnvironment(config);
}

/**
 * Test utilities
 */
export class TestUtils {
  /**
   * Generate random test data
   */
  static generateRandomTestData() {
    const tokenId = Math.floor(Math.random() * 1000000).toString();
    return {
      tokenId,
      name: `Test NFT ${tokenId}`,
      description: `Random test NFT generated for testing purposes`,
      attributes: [
        { trait_type: 'Random', value: Math.random().toString() },
        { trait_type: 'Generated', value: new Date().toISOString() }
      ]
    };
  }

  /**
   * Create deterministic test data
   */
  static createDeterministicTestData(seed: string) {
    return {
      tokenId: `test-${seed}`,
      name: `Deterministic NFT ${seed}`,
      description: `Deterministic test NFT with seed ${seed}`,
      attributes: [
        { trait_type: 'Seed', value: seed },
        { trait_type: 'Deterministic', value: 'true' }
      ]
    };
  }

  /**
   * Validate test result structure
   */
  static validateTestResult(result: any): boolean {
    return (
      result &&
      typeof result.success === 'boolean' &&
      (result.error === undefined || typeof result.error === 'string')
    );
  }

  /**
   * Compare NFT metadata
   */
  static compareNFTMetadata(metadata1: any, metadata2: any): boolean {
    if (!metadata1 || !metadata2) return false;
    
    return (
      metadata1.name === metadata2.name &&
      metadata1.description === metadata2.description &&
      metadata1.image === metadata2.image &&
      JSON.stringify(metadata1.attributes) === JSON.stringify(metadata2.attributes)
    );
  }
}

export default TestEnvironment;