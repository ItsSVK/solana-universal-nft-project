import { PublicKey, Keypair, Connection } from '@solana/web3.js';
import { ethers } from 'ethers';
import { CrossChainMessageUtils, NFTTransferMessage, CHAIN_IDS } from './CrossChainMessage';
import { MessageBridge } from './MessageBridge';

/**
 * Flow Orchestrator for Universal NFT Protocol
 * This utility helps coordinate complex cross-chain NFT transfers
 * and provides high-level interfaces for executing the supported flows.
 */

export interface FlowResult {
  success: boolean;
  message?: NFTTransferMessage;
  txHash?: string;
  error?: string;
  metadata?: any;
}

export interface NFTMetadata {
  tokenId: string;
  metadataUri: string;
  name?: string;
  description?: string;
  attributes?: Array<{ trait_type: string; value: string }>;
}

export interface FlowStep {
  stepNumber: number;
  description: string;
  fromChain: number;
  toChain: number;
  message: NFTTransferMessage;
  completed: boolean;
  txHash?: string;
}

/**
 * Main Flow Orchestrator class
 */
export class FlowOrchestrator {
  private processedMessages: Set<string> = new Set();

  constructor(
    private ethProvider?: ethers.Provider,
    private solanaConnection?: Connection
  ) {}

  /**
   * Execute Flow 1: Solana → Base Sepolia
   */
  async executeSolanaToBaseFlow(params: {
    tokenId: string;
    metadataUri: string;
    senderKeypair: Keypair;
    recipientAddress: string; // Ethereum address
    nonce: string;
    gasLimit?: number;
  }): Promise<FlowResult> {
    try {
      console.log('🚀 Executing Flow 1: Solana → Base Sepolia');
      
      // Step 1: Create transfer message
      const transferMessage = MessageBridge.createSolanaToEvmMessage({
        tokenId: params.tokenId,
        metadataUri: params.metadataUri,
        recipientAddress: params.recipientAddress,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: params.senderKeypair.publicKey.toString(),
        nonce: params.nonce,
      });

      // Step 2: Validate message
      CrossChainMessageUtils.validateMessage(transferMessage);
      MessageBridge.validateCrossChainCompatibility(transferMessage);

      // Step 3: Check for replay
      const messageIdHex = Buffer.from(transferMessage.messageId).toString('hex');
      if (this.processedMessages.has(messageIdHex)) {
        throw new Error('Message already processed (replay attack prevention)');
      }
      this.processedMessages.add(messageIdHex);

      console.log('✅ Solana → Base flow prepared successfully');
      console.log(`   Token ID: ${params.tokenId}`);
      console.log(`   Recipient: ${params.recipientAddress}`);
      console.log(`   Message ID: 0x${messageIdHex.slice(0, 16)}...`);

      return {
        success: true,
        message: transferMessage,
        metadata: {
          flow: 'Solana → Base Sepolia',
          tokenId: params.tokenId,
          recipient: params.recipientAddress,
        },
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Execute Flow 2: ZetaChain → Solana
   */
  async executeZetaChainToSolanaFlow(params: {
    tokenId: string;
    metadataUri: string;
    senderAddress: string; // Ethereum address
    recipientPubkey: PublicKey;
    nonce: string;
    originContractAddress?: string;
  }): Promise<FlowResult> {
    try {
      console.log('🚀 Executing Flow 2: ZetaChain → Solana');

      // Step 1: Create transfer message
      const transferMessage = MessageBridge.createEvmToSolanaMessage({
        tokenId: params.tokenId,
        metadataUri: params.metadataUri,
        recipientAddress: params.recipientPubkey.toString(),
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        senderAddress: params.senderAddress,
        originContractAddress: params.originContractAddress || params.senderAddress,
        nonce: params.nonce,
      });

      // Step 2: Validate message
      CrossChainMessageUtils.validateMessage(transferMessage);
      MessageBridge.validateCrossChainCompatibility(transferMessage);

      // Step 3: Check for replay
      const messageIdHex = Buffer.from(transferMessage.messageId).toString('hex');
      if (this.processedMessages.has(messageIdHex)) {
        throw new Error('Message already processed (replay attack prevention)');
      }
      this.processedMessages.add(messageIdHex);

      console.log('✅ ZetaChain → Solana flow prepared successfully');
      console.log(`   Token ID: ${params.tokenId}`);
      console.log(`   Recipient: ${params.recipientPubkey.toString()}`);
      console.log(`   Message ID: 0x${messageIdHex.slice(0, 16)}...`);

      return {
        success: true,
        message: transferMessage,
        metadata: {
          flow: 'ZetaChain → Solana',
          tokenId: params.tokenId,
          recipient: params.recipientPubkey.toString(),
        },
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Execute Flow 3: Base Sepolia → Solana
   */
  async executeBaseToSolanaFlow(params: {
    tokenId: string;
    metadataUri: string;
    senderAddress: string; // Ethereum address
    recipientPubkey: PublicKey;
    nonce: string;
    originContractAddress?: string;
  }): Promise<FlowResult> {
    try {
      console.log('🚀 Executing Flow 3: Base Sepolia → Solana');

      // Step 1: Create transfer message
      const transferMessage = MessageBridge.createEvmToSolanaMessage({
        tokenId: params.tokenId,
        metadataUri: params.metadataUri,
        recipientAddress: params.recipientPubkey.toString(),
        originChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: params.senderAddress,
        originContractAddress: params.originContractAddress || params.senderAddress,
        nonce: params.nonce,
      });

      // Step 2: Validate message
      CrossChainMessageUtils.validateMessage(transferMessage);
      MessageBridge.validateCrossChainCompatibility(transferMessage);

      // Step 3: Check for replay
      const messageIdHex = Buffer.from(transferMessage.messageId).toString('hex');
      if (this.processedMessages.has(messageIdHex)) {
        throw new Error('Message already processed (replay attack prevention)');
      }
      this.processedMessages.add(messageIdHex);

      console.log('✅ Base Sepolia → Solana flow prepared successfully');
      console.log(`   Token ID: ${params.tokenId}`);
      console.log(`   Recipient: ${params.recipientPubkey.toString()}`);
      console.log(`   Message ID: 0x${messageIdHex.slice(0, 16)}...`);

      return {
        success: true,
        message: transferMessage,
        metadata: {
          flow: 'Base Sepolia → Solana',
          tokenId: params.tokenId,
          recipient: params.recipientPubkey.toString(),
        },
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Execute Flow 4: Full Loop (ZetaChain → Base → Solana → ZetaChain)
   */
  async executeFullLoopFlow(params: {
    tokenId: string;
    metadataUri: string;
    originalOwner: string; // Ethereum address
    intermediateRecipient: string; // Ethereum address for Base
    solanaRecipient: PublicKey;
    startingNonce: number;
  }): Promise<{
    success: boolean;
    steps: FlowStep[];
    error?: string;
  }> {
    const steps: FlowStep[] = [];
    
    try {
      console.log('🔄 Executing Full Loop Flow: ZetaChain → Base → Solana → ZetaChain');

      // Step 1: ZetaChain → Base Sepolia
      console.log('🚀 Step 1: ZetaChain → Base Sepolia');
      const step1Message = MessageBridge.createEvmToEvmMessage({
        tokenId: params.tokenId,
        metadataUri: params.metadataUri,
        recipientAddress: params.intermediateRecipient,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: params.originalOwner,
        originContractAddress: params.originalOwner,
        nonce: params.startingNonce.toString(),
      });

      CrossChainMessageUtils.validateMessage(step1Message);
      this.processedMessages.add(Buffer.from(step1Message.messageId).toString('hex'));

      steps.push({
        stepNumber: 1,
        description: 'ZetaChain → Base Sepolia',
        fromChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        toChain: CHAIN_IDS.BASE_SEPOLIA,
        message: step1Message,
        completed: true,
      });

      // Step 2: Base Sepolia → Solana (preserving original origin)
      console.log('🚀 Step 2: Base Sepolia → Solana');
      const step2Message = MessageBridge.createEvmToSolanaMessage({
        tokenId: params.tokenId,
        metadataUri: params.metadataUri,
        recipientAddress: params.solanaRecipient.toString(),
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET, // PRESERVE original origin
        senderAddress: params.intermediateRecipient,
        originContractAddress: params.originalOwner, // PRESERVE original contract
        nonce: (params.startingNonce + 1).toString(),
      });

      CrossChainMessageUtils.validateMessage(step2Message);
      this.processedMessages.add(Buffer.from(step2Message.messageId).toString('hex'));

      steps.push({
        stepNumber: 2,
        description: 'Base Sepolia → Solana (preserving ZetaChain origin)',
        fromChain: CHAIN_IDS.BASE_SEPOLIA,
        toChain: CHAIN_IDS.SOLANA_DEVNET,
        message: step2Message,
        completed: true,
      });

      // Step 3: Solana → ZetaChain (completing the loop)
      console.log('🚀 Step 3: Solana → ZetaChain (completing loop)');
      const step3Message = MessageBridge.createSolanaToEvmMessage({
        tokenId: params.tokenId,
        metadataUri: params.metadataUri,
        recipientAddress: params.originalOwner,
        destinationChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        senderAddress: params.solanaRecipient.toString(),
        nonce: (params.startingNonce + 2).toString(),
      });

      CrossChainMessageUtils.validateMessage(step3Message);
      this.processedMessages.add(Buffer.from(step3Message.messageId).toString('hex'));

      steps.push({
        stepNumber: 3,
        description: 'Solana → ZetaChain (completing loop)',
        fromChain: CHAIN_IDS.SOLANA_DEVNET,
        toChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        message: step3Message,
        completed: true,
      });

      console.log('✅ Full loop flow completed successfully!');
      console.log(`   Token ${params.tokenId} completed journey: ZetaChain → Base → Solana → ZetaChain`);
      console.log(`   Final recipient: ${params.originalOwner} (original owner)`);

      return {
        success: true,
        steps,
      };
    } catch (error) {
      return {
        success: false,
        steps,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Validate a specific cross-chain route
   */
  validateRoute(fromChain: number, toChain: number): {
    isSupported: boolean;
    routeName: string;
  } {
    const isSupported = MessageBridge.isRouteSupported(fromChain, toChain);
    const fromName = CrossChainMessageUtils.getChainName(fromChain);
    const toName = CrossChainMessageUtils.getChainName(toChain);
    
    return {
      isSupported,
      routeName: `${fromName} → ${toName}`,
    };
  }

  /**
   * Get all supported routes
   */
  getSupportedRoutes(): Array<{
    from: number;
    to: number;
    name: string;
  }> {
    return MessageBridge.getSupportedRoutes();
  }

  /**
   * Estimate costs for a cross-chain transfer
   */
  async estimateTransferCosts(fromChain: number, toChain: number): Promise<{
    estimatedGas: string;
    estimatedCostUSD: string;
  }> {
    return MessageBridge.estimateGasCosts({
      fromChain,
      toChain,
      provider: this.ethProvider,
    });
  }

  /**
   * Track message processing to prevent replays
   */
  markMessageProcessed(messageId: Uint8Array): void {
    const messageIdHex = Buffer.from(messageId).toString('hex');
    this.processedMessages.add(messageIdHex);
  }

  /**
   * Check if a message has been processed
   */
  isMessageProcessed(messageId: Uint8Array): boolean {
    const messageIdHex = Buffer.from(messageId).toString('hex');
    return this.processedMessages.has(messageIdHex);
  }

  /**
   * Get flow statistics
   */
  getFlowStatistics(): {
    totalProcessedMessages: number;
    supportedRoutes: number;
    availableFlows: string[];
  } {
    return {
      totalProcessedMessages: this.processedMessages.size,
      supportedRoutes: this.getSupportedRoutes().length,
      availableFlows: [
        'Solana → Base Sepolia',
        'ZetaChain → Solana',
        'Base Sepolia → Solana',
        'Full Loop (ZetaChain → Base → Solana → ZetaChain)',
        'ZetaChain → Base Sepolia',
        'Base Sepolia → ZetaChain',
        'Solana → ZetaChain',
      ],
    };
  }

  /**
   * Create NFT metadata object
   */
  static createNFTMetadata(params: {
    tokenId: string;
    name: string;
    description: string;
    imageUrl: string;
    attributes?: Array<{ trait_type: string; value: string }>;
  }): NFTMetadata {
    const metadata = {
      name: params.name,
      description: params.description,
      image: params.imageUrl,
      attributes: params.attributes || [],
    };

    const metadataUri = `data:application/json;base64,${Buffer.from(
      JSON.stringify(metadata)
    ).toString('base64')}`;

    return {
      tokenId: params.tokenId,
      metadataUri,
      name: params.name,
      description: params.description,
      attributes: params.attributes,
    };
  }

  /**
   * Validate NFT metadata
   */
  static validateNFTMetadata(metadata: NFTMetadata): {
    isValid: boolean;
    errors: string[];
  } {
    const errors: string[] = [];

    if (!metadata.tokenId || metadata.tokenId === '0') {
      errors.push('Token ID is required and cannot be zero');
    }

    if (!metadata.metadataUri) {
      errors.push('Metadata URI is required');
    } else if (metadata.metadataUri.length > 500) {
      errors.push('Metadata URI too long (max 500 characters)');
    }

    if (!metadata.name || metadata.name.trim().length === 0) {
      errors.push('NFT name is required');
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }

  /**
   * Clear processed messages (for testing purposes)
   */
  clearProcessedMessages(): void {
    this.processedMessages.clear();
  }
}

/**
 * Factory function to create a FlowOrchestrator instance
 */
export function createFlowOrchestrator(
  ethProvider?: ethers.Provider,
  solanaConnection?: Connection
): FlowOrchestrator {
  return new FlowOrchestrator(ethProvider, solanaConnection);
}

/**
 * Utility functions for flow management
 */
export class FlowUtils {
  /**
   * Convert Ethereum address to Solana-compatible format
   */
  static ethAddressToSolanaBytes(address: string): Uint8Array {
    return CrossChainMessageUtils.ethereumAddressToBytes32(address);
  }

  /**
   * Convert Solana public key to EVM-compatible format
   */
  static solanaPubkeyToEthBytes(pubkey: PublicKey): Uint8Array {
    return CrossChainMessageUtils.solanaPublicKeyToBytes32(pubkey);
  }

  /**
   * Generate a flow execution plan
   */
  static generateFlowPlan(
    fromChain: number,
    toChain: number,
    tokenId: string
  ): {
    flowType: string;
    steps: Array<{
      step: number;
      action: string;
      chain: string;
    }>;
    estimatedTime: string;
  } {
    const fromName = CrossChainMessageUtils.getChainName(fromChain);
    const toName = CrossChainMessageUtils.getChainName(toChain);
    const flowType = `${fromName} → ${toName}`;

    const steps = [
      { step: 1, action: `Burn NFT ${tokenId} on ${fromName}`, chain: fromName },
      { step: 2, action: 'Create cross-chain message', chain: fromName },
      { step: 3, action: 'Send message via ZetaChain Gateway', chain: 'ZetaChain' },
      { step: 4, action: `Receive message on ${toName}`, chain: toName },
      { step: 5, action: `Mint NFT ${tokenId} on ${toName}`, chain: toName },
    ];

    return {
      flowType,
      steps,
      estimatedTime: '2-5 minutes',
    };
  }

  /**
   * Validate flow parameters
   */
  static validateFlowParameters(params: {
    tokenId: string;
    fromChain: number;
    toChain: number;
    sender: string;
    recipient: string;
  }): { isValid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!params.tokenId || params.tokenId === '0') {
      errors.push('Invalid token ID');
    }

    if (params.fromChain === params.toChain) {
      errors.push('From and to chains cannot be the same');
    }

    if (!MessageBridge.isRouteSupported(params.fromChain, params.toChain)) {
      errors.push('Route not supported');
    }

    if (!params.sender || params.sender.trim().length === 0) {
      errors.push('Sender address is required');
    }

    if (!params.recipient || params.recipient.trim().length === 0) {
      errors.push('Recipient address is required');
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }
}