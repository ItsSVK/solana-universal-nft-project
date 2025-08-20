import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { UniversalNft } from '../target/types/universal_nft';
import { PublicKey, Keypair } from '@solana/web3.js';
import { expect } from 'chai';
import {
  CrossChainNftMessage,
  MESSAGE_TYPES,
  CHAIN_IDS,
  MESSAGE_CONSTANTS,
  createMintMessage,
  createBurnMessage,
  validateMessage,
  isMintMessage,
  isBurnMessage,
  serializeMessage,
  deserializeMessage,
  generateMessageId,
  publicKeyToBytes,
} from './types/cross-chain-message';

anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.universalNft as Program<UniversalNft>;

describe('Cross-Chain Message Format', () => {
  describe('Message Creation', () => {
    it('should create a valid mint message', () => {
      const tokenId = 123;
      const metadataUri = 'https://example.com/metadata.json';
      const name = 'Test NFT';
      const symbol = 'TNFT';
      const originChainId = CHAIN_IDS.SOLANA;
      const originAddress = new Uint8Array(32).fill(1);
      const recipientAddress = new Uint8Array(32).fill(2);
      const messageId = generateMessageId();

      const message = createMintMessage(
        tokenId,
        metadataUri,
        name,
        symbol,
        originChainId,
        originAddress,
        recipientAddress,
        messageId
      );

      expect(message.messageType).to.equal(MESSAGE_TYPES.NFT_MINT);
      expect(message.tokenId).to.equal(tokenId);
      expect(message.metadataUri).to.equal(metadataUri);
      expect(message.name).to.equal(name);
      expect(message.symbol).to.equal(symbol);
      expect(message.originChainId).to.equal(originChainId);
      expect(message.originAddress).to.deep.equal(originAddress);
      expect(message.recipientAddress).to.deep.equal(recipientAddress);
      expect(message.messageId).to.deep.equal(messageId);
      expect(message.timestamp).to.be.a('number');
      expect(message.timestamp).to.be.greaterThan(0);
    });

    it('should create a valid burn message', () => {
      const tokenId = 456;
      const originChainId = CHAIN_IDS.BASE_SEPOLIA;
      const originAddress = new Uint8Array(32).fill(3);
      const recipientAddress = new Uint8Array(32).fill(4);
      const messageId = generateMessageId();

      const message = createBurnMessage(
        tokenId,
        originChainId,
        originAddress,
        recipientAddress,
        messageId
      );

      expect(message.messageType).to.equal(MESSAGE_TYPES.NFT_BURN);
      expect(message.tokenId).to.equal(tokenId);
      expect(message.metadataUri).to.equal('');
      expect(message.name).to.equal('');
      expect(message.symbol).to.equal('');
      expect(message.originChainId).to.equal(originChainId);
      expect(message.originAddress).to.deep.equal(originAddress);
      expect(message.recipientAddress).to.deep.equal(recipientAddress);
      expect(message.messageId).to.deep.equal(messageId);
      expect(message.timestamp).to.be.a('number');
      expect(message.timestamp).to.be.greaterThan(0);
    });
  });

  describe('Message Validation', () => {
    it('should validate a correct mint message', () => {
      const message = createMintMessage(
        123,
        'https://example.com/metadata.json',
        'Test NFT',
        'TNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const errors = validateMessage(message);
      expect(errors).to.be.an('array');
      expect(errors).to.have.length(0);
    });

    it('should validate a correct burn message', () => {
      const message = createBurnMessage(
        456,
        CHAIN_IDS.BASE_SEPOLIA,
        new Uint8Array(32).fill(3),
        new Uint8Array(32).fill(4),
        generateMessageId()
      );

      const errors = validateMessage(message);
      expect(errors).to.be.an('array');
      expect(errors).to.have.length(0);
    });

    it('should reject message with invalid message type', () => {
      const message: CrossChainNftMessage = {
        messageType: 99, // Invalid type
        tokenId: 123,
        metadataUri: 'https://example.com/metadata.json',
        name: 'Test NFT',
        symbol: 'TNFT',
        originChainId: CHAIN_IDS.SOLANA,
        originAddress: new Uint8Array(32).fill(1),
        recipientAddress: new Uint8Array(32).fill(2),
        timestamp: Math.floor(Date.now() / 1000),
        messageId: generateMessageId(),
      };

      const errors = validateMessage(message);
      expect(errors).to.include('Invalid message type - must be 1-4');
    });

    it('should reject message with invalid token ID', () => {
      const message = createMintMessage(
        0, // Invalid token ID
        'https://example.com/metadata.json',
        'Test NFT',
        'TNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const errors = validateMessage(message);
      expect(errors).to.include('Invalid token ID - must be greater than 0');
    });

    it('should reject mint message with empty metadata URI', () => {
      const message = createMintMessage(
        123,
        '', // Empty metadata URI
        'Test NFT',
        'TNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const errors = validateMessage(message);
      expect(errors).to.include('Invalid metadata URI - cannot be empty');
    });

    it('should reject mint message with empty name', () => {
      const message = createMintMessage(
        123,
        'https://example.com/metadata.json',
        '', // Empty name
        'TNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const errors = validateMessage(message);
      expect(errors).to.include('Invalid NFT name - cannot be empty');
    });

    it('should reject mint message with empty symbol', () => {
      const message = createMintMessage(
        123,
        'https://example.com/metadata.json',
        'Test NFT',
        '', // Empty symbol
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const errors = validateMessage(message);
      expect(errors).to.include('Invalid NFT symbol - cannot be empty');
    });

    it('should reject message with unsupported chain ID', () => {
      const message = createMintMessage(
        123,
        'https://example.com/metadata.json',
        'Test NFT',
        'TNFT',
        99, // Unsupported chain ID
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const errors = validateMessage(message);
      expect(errors).to.include('Unsupported chain ID');
    });

    it('should reject message with old timestamp', () => {
      const message = createMintMessage(
        123,
        'https://example.com/metadata.json',
        'Test NFT',
        'TNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      // Set timestamp to 2 hours ago (older than MAX_MESSAGE_AGE_SECONDS)
      message.timestamp =
        Math.floor(Date.now() / 1000) -
        (MESSAGE_CONSTANTS.MAX_MESSAGE_AGE_SECONDS + 3600);

      const errors = validateMessage(message);
      expect(errors).to.include(
        'Message too old - timestamp exceeds maximum age'
      );
    });

    it('should reject message with invalid address lengths', () => {
      const message = createMintMessage(
        123,
        'https://example.com/metadata.json',
        'Test NFT',
        'TNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(16).fill(1), // Wrong length
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const errors = validateMessage(message);
      expect(errors).to.include(
        'Invalid origin address length - must be 32 bytes'
      );
    });

    it('should reject message with invalid message ID length', () => {
      const message = createMintMessage(
        123,
        'https://example.com/metadata.json',
        'Test NFT',
        'TNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        new Uint8Array(16).fill(1) // Wrong length
      );

      const errors = validateMessage(message);
      expect(errors).to.include('Invalid message ID length - must be 32 bytes');
    });
  });

  describe('Message Type Checks', () => {
    it('should correctly identify mint messages', () => {
      const mintMessage = createMintMessage(
        123,
        'https://example.com/metadata.json',
        'Test NFT',
        'TNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      expect(isMintMessage(mintMessage)).to.be.true;
      expect(isBurnMessage(mintMessage)).to.be.false;
    });

    it('should correctly identify burn messages', () => {
      const burnMessage = createBurnMessage(
        456,
        CHAIN_IDS.BASE_SEPOLIA,
        new Uint8Array(32).fill(3),
        new Uint8Array(32).fill(4),
        generateMessageId()
      );

      expect(isBurnMessage(burnMessage)).to.be.true;
      expect(isMintMessage(burnMessage)).to.be.false;
    });
  });

  describe('Message Serialization and Deserialization', () => {
    it('should serialize and deserialize a mint message correctly', () => {
      const originalMessage = createMintMessage(
        123,
        'https://example.com/metadata.json',
        'Test NFT',
        'TNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const serialized = serializeMessage(originalMessage);
      const deserialized = deserializeMessage(serialized);

      expect(deserialized.messageType).to.equal(originalMessage.messageType);
      expect(deserialized.tokenId).to.equal(originalMessage.tokenId);
      expect(deserialized.metadataUri).to.equal(originalMessage.metadataUri);
      expect(deserialized.name).to.equal(originalMessage.name);
      expect(deserialized.symbol).to.equal(originalMessage.symbol);
      expect(deserialized.originChainId).to.equal(
        originalMessage.originChainId
      );
      expect(deserialized.originAddress).to.deep.equal(
        originalMessage.originAddress
      );
      expect(deserialized.recipientAddress).to.deep.equal(
        originalMessage.recipientAddress
      );
      expect(deserialized.messageId).to.deep.equal(originalMessage.messageId);
      expect(deserialized.timestamp).to.equal(originalMessage.timestamp);
    });

    it('should serialize and deserialize a burn message correctly', () => {
      const originalMessage = createBurnMessage(
        456,
        CHAIN_IDS.BASE_SEPOLIA,
        new Uint8Array(32).fill(3),
        new Uint8Array(32).fill(4),
        generateMessageId()
      );

      const serialized = serializeMessage(originalMessage);
      const deserialized = deserializeMessage(serialized);

      expect(deserialized.messageType).to.equal(originalMessage.messageType);
      expect(deserialized.tokenId).to.equal(originalMessage.tokenId);
      expect(deserialized.metadataUri).to.equal(originalMessage.metadataUri);
      expect(deserialized.name).to.equal(originalMessage.name);
      expect(deserialized.symbol).to.equal(originalMessage.symbol);
      expect(deserialized.originChainId).to.equal(
        originalMessage.originChainId
      );
      expect(deserialized.originAddress).to.deep.equal(
        originalMessage.originAddress
      );
      expect(deserialized.recipientAddress).to.deep.equal(
        originalMessage.recipientAddress
      );
      expect(deserialized.messageId).to.deep.equal(originalMessage.messageId);
      expect(deserialized.timestamp).to.equal(originalMessage.timestamp);
    });

    it('should handle messages with additional metadata', () => {
      const additionalMetadata = new Uint8Array([1, 2, 3, 4, 5]);
      const originalMessage = createMintMessage(
        123,
        'https://example.com/metadata.json',
        'Test NFT',
        'TNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );
      originalMessage.additionalMetadata = additionalMetadata;

      const serialized = serializeMessage(originalMessage);
      const deserialized = deserializeMessage(serialized);

      expect(deserialized.additionalMetadata).to.deep.equal(additionalMetadata);
    });
  });

  describe('Utility Functions', () => {
    it('should generate unique message IDs', () => {
      const messageId1 = generateMessageId();
      const messageId2 = generateMessageId();

      expect(messageId1).to.have.length(32);
      expect(messageId2).to.have.length(32);
      expect(messageId1).to.not.deep.equal(messageId2);
    });

    it('should convert public key to bytes correctly', () => {
      const keypair = Keypair.generate();
      const bytes = publicKeyToBytes(keypair.publicKey);

      expect(bytes).to.have.length(32);
      expect(bytes).to.deep.equal(new Uint8Array(keypair.publicKey.toBytes()));
    });
  });

  describe('Constants', () => {
    it('should have correct message type constants', () => {
      expect(MESSAGE_TYPES.NFT_MINT).to.equal(1);
      expect(MESSAGE_TYPES.NFT_BURN).to.equal(2);
      expect(MESSAGE_TYPES.NFT_TRANSFER).to.equal(3);
      expect(MESSAGE_TYPES.COLLECTION_UPDATE).to.equal(4);
    });

    it('should have correct chain ID constants', () => {
      expect(CHAIN_IDS.SOLANA).to.equal(1);
      expect(CHAIN_IDS.BASE_SEPOLIA).to.equal(2);
      expect(CHAIN_IDS.BNB_TESTNET).to.equal(3);
    });

    it('should have correct message format constants', () => {
      expect(MESSAGE_CONSTANTS.MAX_MESSAGE_AGE_SECONDS).to.equal(3600);
      expect(MESSAGE_CONSTANTS.MAX_METADATA_URI_LENGTH).to.equal(200);
      expect(MESSAGE_CONSTANTS.MAX_NFT_NAME_LENGTH).to.equal(32);
      expect(MESSAGE_CONSTANTS.MAX_NFT_SYMBOL_LENGTH).to.equal(10);
      expect(MESSAGE_CONSTANTS.MAX_ADDITIONAL_METADATA_SIZE).to.equal(1024);
      expect(MESSAGE_CONSTANTS.MIN_MESSAGE_SIZE).to.equal(100);
      expect(MESSAGE_CONSTANTS.MAX_MESSAGE_SIZE).to.equal(2048);
    });
  });
});
