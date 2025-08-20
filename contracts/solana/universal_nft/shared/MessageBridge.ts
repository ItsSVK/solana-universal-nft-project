import { PublicKey } from '@solana/web3.js';
import { ethers } from 'ethers';
import { CrossChainMessageUtils, NFTTransferMessage, CHAIN_IDS } from './CrossChainMessage';

/**
 * Bridge utility for converting between different message formats
 * This module handles conversion between the shared CrossChainMessage format
 * and various chain-specific formats (Solana, EVM, etc.)
 */

/**
 * Solana-specific message format (matches the Rust program structure)
 */
export interface SolanaNftMessage {
  messageType: number;           // u8
  tokenId: string;              // u64 as string
  metadataUri: string;          // String
  name: string;                 // String  
  symbol: string;               // String
  originChainId: number;        // u8
  originAddress: Uint8Array;    // [u8; 32]
  recipientAddress: Uint8Array; // [u8; 32]
  timestamp: number;            // i64
  messageId: Uint8Array;        // [u8; 32]
  additionalMetadata?: Uint8Array; // Option<Vec<u8>>
}

/**
 * Message Bridge class for format conversions
 */
export class MessageBridge {
  
  /**
   * Convert shared NFTTransferMessage to Solana's internal format
   */
  static toSolanaFormat(message: NFTTransferMessage): SolanaNftMessage {
    return {
      messageType: 1, // NFT_MINT
      tokenId: message.tokenId,
      metadataUri: message.metadataUri,
      name: this.extractNameFromUri(message.metadataUri),
      symbol: "UNFT", // Default symbol
      originChainId: message.originChain,
      originAddress: message.originContract,
      recipientAddress: message.recipient,
      timestamp: message.timestamp,
      messageId: message.messageId,
      additionalMetadata: undefined,
    };
  }

  /**
   * Convert Solana format to shared NFTTransferMessage
   */
  static fromSolanaFormat(message: SolanaNftMessage): NFTTransferMessage {
    return {
      tokenId: message.tokenId,
      metadataUri: message.metadataUri,
      recipient: message.recipientAddress,
      originChain: message.originChainId,
      destinationChain: 0, // Will be set by caller
      messageId: message.messageId,
      timestamp: message.timestamp,
      originContract: message.originAddress,
      nonce: "0", // Will be set by caller
    };
  }

  /**
   * Convert shared NFTTransferMessage to EVM ABI format
   * This creates properly ABI-encoded data for Solidity contracts
   */
  static toEvmFormat(message: NFTTransferMessage): string {
    return ethers.AbiCoder.defaultAbiCoder().encode(
      [
        'uint256',  // tokenId
        'string',   // metadataUri
        'bytes32',  // recipient
        'uint32',   // originChain
        'uint32',   // destinationChain
        'bytes32',  // messageId
        'uint64',   // timestamp
        'bytes32',  // originContract
        'uint256',  // nonce
      ],
      [
        message.tokenId,
        message.metadataUri,
        message.recipient,
        message.originChain,
        message.destinationChain,
        message.messageId,
        message.timestamp,
        message.originContract,
        message.nonce,
      ]
    );
  }

  /**
   * Convert EVM ABI format to shared NFTTransferMessage
   */
  static fromEvmFormat(encodedData: string): NFTTransferMessage {
    const decoded = ethers.AbiCoder.defaultAbiCoder().decode(
      [
        'uint256',  // tokenId
        'string',   // metadataUri
        'bytes32',  // recipient
        'uint32',   // originChain
        'uint32',   // destinationChain
        'bytes32',  // messageId
        'uint64',   // timestamp
        'bytes32',  // originContract
        'uint256',  // nonce
      ],
      encodedData
    );

    return {
      tokenId: decoded[0].toString(),
      metadataUri: decoded[1],
      recipient: ethers.getBytes(decoded[2]),
      originChain: decoded[3],
      destinationChain: decoded[4],
      messageId: ethers.getBytes(decoded[5]),
      timestamp: Number(decoded[6]),
      originContract: ethers.getBytes(decoded[7]),
      nonce: decoded[8].toString(),
    };
  }

  /**
   * Create a cross-chain message for NFT transfer from Solana
   */
  static createSolanaToEvmMessage(params: {
    tokenId: string;
    metadataUri: string;
    recipientAddress: string; // EVM address
    destinationChain: number;
    senderAddress: string; // Solana address
    nonce: string;
  }): NFTTransferMessage {
    const timestamp = Math.floor(Date.now() / 1000);
    const senderPubkey = new PublicKey(params.senderAddress);
    const recipientBytes32 = CrossChainMessageUtils.ethereumAddressToBytes32(params.recipientAddress);
    const originContractBytes32 = CrossChainMessageUtils.solanaPublicKeyToBytes32(senderPubkey);

    const messageId = CrossChainMessageUtils.generateMessageId(
      params.senderAddress,
      params.tokenId,
      params.destinationChain,
      params.nonce,
      timestamp
    );

    return {
      tokenId: params.tokenId,
      metadataUri: params.metadataUri,
      recipient: recipientBytes32,
      originChain: CHAIN_IDS.SOLANA_DEVNET,
      destinationChain: params.destinationChain,
      messageId,
      timestamp,
      originContract: originContractBytes32,
      nonce: params.nonce,
    };
  }

  /**
   * Create a cross-chain message for NFT transfer from EVM to Solana
   */
  static createEvmToSolanaMessage(params: {
    tokenId: string;
    metadataUri: string;
    recipientAddress: string; // Solana address
    originChain: number;
    senderAddress: string; // EVM address
    originContractAddress: string; // EVM contract address
    nonce: string;
  }): NFTTransferMessage {
    const timestamp = Math.floor(Date.now() / 1000);
    const recipientPubkey = new PublicKey(params.recipientAddress);
    const recipientBytes32 = CrossChainMessageUtils.solanaPublicKeyToBytes32(recipientPubkey);
    const originContractBytes32 = CrossChainMessageUtils.ethereumAddressToBytes32(params.originContractAddress);

    const messageId = CrossChainMessageUtils.generateMessageId(
      params.senderAddress,
      params.tokenId,
      CHAIN_IDS.SOLANA_DEVNET,
      params.nonce,
      timestamp
    );

    return {
      tokenId: params.tokenId,
      metadataUri: params.metadataUri,
      recipient: recipientBytes32,
      originChain: params.originChain,
      destinationChain: CHAIN_IDS.SOLANA_DEVNET,
      messageId,
      timestamp,
      originContract: originContractBytes32,
      nonce: params.nonce,
    };
  }

  /**
   * Create a cross-chain message for NFT transfer between EVM chains
   */
  static createEvmToEvmMessage(params: {
    tokenId: string;
    metadataUri: string;
    recipientAddress: string; // EVM address
    originChain: number;
    destinationChain: number;
    senderAddress: string; // EVM address
    originContractAddress: string; // EVM contract address
    nonce: string;
  }): NFTTransferMessage {
    const timestamp = Math.floor(Date.now() / 1000);
    const recipientBytes32 = CrossChainMessageUtils.ethereumAddressToBytes32(params.recipientAddress);
    const originContractBytes32 = CrossChainMessageUtils.ethereumAddressToBytes32(params.originContractAddress);

    const messageId = CrossChainMessageUtils.generateMessageId(
      params.senderAddress,
      params.tokenId,
      params.destinationChain,
      params.nonce,
      timestamp
    );

    return {
      tokenId: params.tokenId,
      metadataUri: params.metadataUri,
      recipient: recipientBytes32,
      originChain: params.originChain,
      destinationChain: params.destinationChain,
      messageId,
      timestamp,
      originContract: originContractBytes32,
      nonce: params.nonce,
    };
  }

  /**
   * Validate cross-chain message format compatibility
   */
  static validateCrossChainCompatibility(message: NFTTransferMessage): void {
    // Check token ID is valid
    const tokenIdBN = BigInt(message.tokenId);
    if (tokenIdBN === 0n) {
      throw new Error('Invalid token ID: cannot be zero');
    }

    // Check metadata URI is not empty and not too long
    if (!message.metadataUri || message.metadataUri.length === 0) {
      throw new Error('Metadata URI cannot be empty');
    }
    if (message.metadataUri.length > 500) {
      throw new Error('Metadata URI too long (max 500 characters)');
    }

    // Check timestamp is reasonable (not too old, not too far in future)
    const currentTime = Math.floor(Date.now() / 1000);
    const timeDiff = Math.abs(message.timestamp - currentTime);
    if (timeDiff > 86400) { // 24 hours
      throw new Error('Message timestamp too old or too far in future');
    }

    // Check origin and destination chains are different
    if (message.originChain === message.destinationChain) {
      throw new Error('Origin and destination chains cannot be the same');
    }

    // Check message ID is not all zeros
    const allZeros = new Uint8Array(32);
    if (Buffer.from(message.messageId).equals(Buffer.from(allZeros))) {
      throw new Error('Message ID cannot be all zeros');
    }

    // Check addresses are properly formatted
    if (message.recipient.length !== 32) {
      throw new Error('Recipient address must be 32 bytes');
    }
    if (message.originContract.length !== 32) {
      throw new Error('Origin contract address must be 32 bytes');
    }
  }

  /**
   * Extract NFT name from metadata URI (simplified implementation)
   * In production, you might want to fetch the metadata and extract the name
   */
  private static extractNameFromUri(uri: string): string {
    // This is a simplified implementation
    // You could enhance this to actually fetch the metadata JSON
    const fileName = uri.split('/').pop() || 'Universal NFT';
    return fileName.replace(/\.[^/.]+$/, ''); // Remove file extension
  }

  /**
   * Get supported transfer routes
   */
  static getSupportedRoutes(): Array<{from: number, to: number, name: string}> {
    return [
      { from: CHAIN_IDS.ZETACHAIN_TESTNET, to: CHAIN_IDS.BASE_SEPOLIA, name: 'ZetaChain → Base Sepolia' },
      { from: CHAIN_IDS.ZETACHAIN_TESTNET, to: CHAIN_IDS.SOLANA_DEVNET, name: 'ZetaChain → Solana Devnet' },
      { from: CHAIN_IDS.BASE_SEPOLIA, to: CHAIN_IDS.ZETACHAIN_TESTNET, name: 'Base Sepolia → ZetaChain' },
      { from: CHAIN_IDS.BASE_SEPOLIA, to: CHAIN_IDS.SOLANA_DEVNET, name: 'Base Sepolia → Solana Devnet' },
      { from: CHAIN_IDS.SOLANA_DEVNET, to: CHAIN_IDS.ZETACHAIN_TESTNET, name: 'Solana Devnet → ZetaChain' },
      { from: CHAIN_IDS.SOLANA_DEVNET, to: CHAIN_IDS.BASE_SEPOLIA, name: 'Solana Devnet → Base Sepolia' },
    ];
  }

  /**
   * Check if a transfer route is supported
   */
  static isRouteSupported(fromChain: number, toChain: number): boolean {
    return this.getSupportedRoutes().some(route => 
      route.from === fromChain && route.to === toChain
    );
  }

  /**
   * Estimate gas costs for cross-chain transfer (placeholder implementation)
   */
  static async estimateGasCosts(params: {
    fromChain: number;
    toChain: number;
    provider?: ethers.Provider;
  }): Promise<{
    estimatedGas: string;
    estimatedCostUSD: string;
  }> {
    // This is a placeholder implementation
    // In production, you would integrate with actual gas estimation APIs
    
    const baseGas: Record<number, string> = {
      [CHAIN_IDS.ZETACHAIN_TESTNET]: '100000',
      [CHAIN_IDS.BASE_SEPOLIA]: '150000', 
      [CHAIN_IDS.SOLANA_DEVNET]: '5000', // Different units for Solana
    };

    const estimatedGas = baseGas[params.toChain] || '100000';
    const estimatedCostUSD = '0.50'; // Placeholder

    return {
      estimatedGas,
      estimatedCostUSD,
    };
  }
}

/**
 * Helper functions for testing and development
 */
export class MessageBridgeTestUtils {
  
  /**
   * Create a mock NFT transfer message for testing
   */
  static createMockMessage(overrides: Partial<NFTTransferMessage> = {}): NFTTransferMessage {
    const defaultMessage: NFTTransferMessage = {
      tokenId: "123",
      metadataUri: "https://example.com/metadata/123.json",
      recipient: new Uint8Array(32).fill(1),
      originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
      destinationChain: CHAIN_IDS.BASE_SEPOLIA,
      messageId: new Uint8Array(32).fill(2),
      timestamp: Math.floor(Date.now() / 1000),
      originContract: new Uint8Array(32).fill(3),
      nonce: "1",
    };

    return { ...defaultMessage, ...overrides };
  }

  /**
   * Generate test vectors for cross-chain compatibility testing
   */
  static generateTestVectors(): Array<{
    name: string;
    message: NFTTransferMessage;
    expectedSolanaFormat: SolanaNftMessage;
    expectedEvmFormat: string;
  }> {
    const testMessage = this.createMockMessage();
    
    return [{
      name: "Basic NFT Transfer",
      message: testMessage,
      expectedSolanaFormat: MessageBridge.toSolanaFormat(testMessage),
      expectedEvmFormat: MessageBridge.toEvmFormat(testMessage),
    }];
  }

  /**
   * Validate round-trip conversion consistency
   */
  static validateRoundTrip(originalMessage: NFTTransferMessage): {
    isValid: boolean;
    errors: string[];
  } {
    const errors: string[] = [];

    try {
      // Test EVM round-trip
      const evmEncoded = MessageBridge.toEvmFormat(originalMessage);
      const evmDecoded = MessageBridge.fromEvmFormat(evmEncoded);
      
      if (originalMessage.tokenId !== evmDecoded.tokenId) {
        errors.push('EVM round-trip failed: tokenId mismatch');
      }
      if (originalMessage.metadataUri !== evmDecoded.metadataUri) {
        errors.push('EVM round-trip failed: metadataUri mismatch');
      }
      // Add more comparisons as needed...

      // Test Solana round-trip
      const solanaFormat = MessageBridge.toSolanaFormat(originalMessage);
      const solanaDecoded = MessageBridge.fromSolanaFormat(solanaFormat);
      
      if (originalMessage.tokenId !== solanaDecoded.tokenId) {
        errors.push('Solana round-trip failed: tokenId mismatch');
      }
      // Add more comparisons as needed...

    } catch (error) {
      errors.push(`Round-trip validation error: ${error instanceof Error ? error.message : String(error)}`);
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }
}