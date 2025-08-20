import { PublicKey } from '@solana/web3.js';
import { ethers } from 'ethers';

/**
 * Shared message format for Universal NFT Protocol cross-chain transfers
 * Compatible with both Solana (Rust) and EVM (Solidity) implementations
 */
export interface NFTTransferMessage {
  tokenId: string;         // NFT token ID (as string to handle large numbers)
  metadataUri: string;     // IPFS/HTTP URL to NFT metadata
  recipient: Uint8Array;   // Recipient address (32 bytes to support both EVM and Solana)
  originChain: number;     // Chain ID where NFT was originally minted
  destinationChain: number; // Target chain ID
  messageId: Uint8Array;   // Unique message identifier for replay protection (32 bytes)
  timestamp: number;       // Block timestamp when message was created
  originContract: Uint8Array; // Original contract address where NFT was first minted (32 bytes)
  nonce: string;          // Sender's nonce for additional uniqueness (as string to handle large numbers)
}

/**
 * Chain IDs for supported networks
 */
export const CHAIN_IDS = {
  SOLANA_DEVNET: 900,      // Custom ID for Solana devnet
  ZETACHAIN_TESTNET: 7001,
  BASE_SEPOLIA: 84532,
} as const;

/**
 * Maximum message age in seconds (24 hours)
 */
export const MAX_MESSAGE_AGE = 24 * 60 * 60;

/**
 * Maximum metadata URI length
 */
export const MAX_METADATA_URI_LENGTH = 500;

/**
 * Utility class for handling cross-chain message operations
 */
export class CrossChainMessageUtils {
  /**
   * Generate unique message ID
   */
  static generateMessageId(
    sender: string, // Address as hex string
    tokenId: string,
    destinationChain: number,
    nonce: string,
    timestamp: number
  ): Uint8Array {
    const data = ethers.concat([
      sender,
      ethers.zeroPadValue(ethers.toBeHex(BigInt(tokenId)), 32),
      ethers.zeroPadValue(ethers.toBeHex(destinationChain), 4),
      ethers.zeroPadValue(ethers.toBeHex(BigInt(nonce)), 32),
      ethers.zeroPadValue(ethers.toBeHex(timestamp), 8),
    ]);
    
    return ethers.getBytes(ethers.keccak256(data));
  }

  /**
   * Encode message for cross-chain transmission (Solidity-compatible)
   */
  static encodeForEVM(message: NFTTransferMessage): string {
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
   * Decode message from cross-chain transmission (Solidity-compatible)
   */
  static decodeFromEVM(encodedData: string): NFTTransferMessage {
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
   * Validate message format and constraints
   */
  static validateMessage(message: NFTTransferMessage, currentTimestamp?: number): void {
    const now = currentTimestamp || Math.floor(Date.now() / 1000);

    if (message.timestamp <= 0) {
      throw new Error('Invalid timestamp');
    }

    if (now - message.timestamp > MAX_MESSAGE_AGE) {
      throw new Error('Message too old');
    }

    if (!message.metadataUri || message.metadataUri.length === 0) {
      throw new Error('Empty metadata URI');
    }

    if (message.metadataUri.length > MAX_METADATA_URI_LENGTH) {
      throw new Error('Metadata URI too long');
    }

    if (message.originChain === message.destinationChain) {
      throw new Error('Same chain transfer');
    }

    if (message.recipient.length !== 32) {
      throw new Error('Invalid recipient length');
    }

    if (message.messageId.length !== 32) {
      throw new Error('Invalid message ID length');
    }

    if (message.originContract.length !== 32) {
      throw new Error('Invalid origin contract length');
    }
  }

  /**
   * Convert Ethereum address (20 bytes) to 32-byte format
   */
  static ethereumAddressToBytes32(address: string): Uint8Array {
    const addressBytes = ethers.getBytes(address);
    const result = new Uint8Array(32);
    result.set(addressBytes, 12); // Pad with zeros at the beginning
    return result;
  }

  /**
   * Convert 32-byte format to Ethereum address (20 bytes)
   */
  static bytes32ToEthereumAddress(bytes32: Uint8Array): string {
    return ethers.hexlify(bytes32.slice(12));
  }

  /**
   * Convert Solana public key to 32-byte format
   */
  static solanaPublicKeyToBytes32(publicKey: PublicKey | string): Uint8Array {
    if (typeof publicKey === 'string') {
      publicKey = new PublicKey(publicKey);
    }
    return publicKey.toBytes();
  }

  /**
   * Convert 32-byte format to Solana public key
   */
  static bytes32ToSolanaPublicKey(bytes32: Uint8Array): PublicKey {
    return new PublicKey(bytes32);
  }

  /**
   * Create a new NFT transfer message
   */
  static createMessage(params: {
    tokenId: string;
    metadataUri: string;
    recipient: string; // Address as string (will be converted to bytes32)
    originChain: number;
    destinationChain: number;
    originContract: string; // Address as string (will be converted to bytes32)
    sender: string; // Sender address for message ID generation
    nonce: string;
    isRecipientSolana?: boolean; // Whether recipient is Solana address
    isOriginContractSolana?: boolean; // Whether origin contract is Solana address
  }): NFTTransferMessage {
    const timestamp = Math.floor(Date.now() / 1000);
    
    // Convert addresses to bytes32 format
    const recipientBytes32 = params.isRecipientSolana 
      ? CrossChainMessageUtils.solanaPublicKeyToBytes32(params.recipient)
      : CrossChainMessageUtils.ethereumAddressToBytes32(params.recipient);

    const originContractBytes32 = params.isOriginContractSolana
      ? CrossChainMessageUtils.solanaPublicKeyToBytes32(params.originContract)
      : CrossChainMessageUtils.ethereumAddressToBytes32(params.originContract);

    const messageId = CrossChainMessageUtils.generateMessageId(
      params.sender,
      params.tokenId,
      params.destinationChain,
      params.nonce,
      timestamp
    );

    const message: NFTTransferMessage = {
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

    // Validate the created message
    CrossChainMessageUtils.validateMessage(message, timestamp);

    return message;
  }

  /**
   * Get chain name from chain ID
   */
  static getChainName(chainId: number): string {
    switch (chainId) {
      case CHAIN_IDS.SOLANA_DEVNET:
        return 'Solana Devnet';
      case CHAIN_IDS.ZETACHAIN_TESTNET:
        return 'ZetaChain Testnet';
      case CHAIN_IDS.BASE_SEPOLIA:
        return 'Base Sepolia';
      default:
        return `Unknown Chain (${chainId})`;
    }
  }
}

// Export types and constants
export type ChainId = typeof CHAIN_IDS[keyof typeof CHAIN_IDS];
export { NFTTransferMessage as CrossChainMessage };