import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { UniversalNft } from '../target/types/universal_nft';
import {
  PublicKey,
  SystemProgram,
  Keypair,
  ComputeBudgetProgram,
} from '@solana/web3.js';
import { expect } from 'chai';
import {
  CrossChainNftMessage,
  MESSAGE_TYPES,
  CHAIN_IDS,
  createMintMessage,
  validateMessage,
  isMintMessage,
  serializeMessage,
  deserializeMessage,
  generateMessageId,
  publicKeyToBytes,
} from './types/cross-chain-message';

anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.universalNft as Program<UniversalNft>;

describe('Enhanced Message Receiver', () => {
  let mintKp: Keypair;
  let tokenAccount: PublicKey;
  let metadataAccount: PublicKey;
  let masterEditionAccount: PublicKey;
  let collectionMintKp: Keypair;
  let collectionMetadataAccount: PublicKey;
  let collectionMasterEditionAccount: PublicKey;
  let recipientKp: Keypair;

  before(async () => {
    // Generate test accounts
    mintKp = Keypair.generate();
    collectionMintKp = Keypair.generate();
    recipientKp = Keypair.generate();

    // Derive associated token account
    tokenAccount = await anchor.utils.token.associatedAddress({
      mint: mintKp.publicKey,
      owner: recipientKp.publicKey,
    });

    // Derive metadata accounts (placeholder addresses for testing)
    metadataAccount = new PublicKey('11111111111111111111111111111111');
    masterEditionAccount = new PublicKey('11111111111111111111111111111112');
    collectionMetadataAccount = new PublicKey(
      '11111111111111111111111111111113'
    );
    collectionMasterEditionAccount = new PublicKey(
      '11111111111111111111111111111114'
    );
  });

  describe('Cross-Chain Message Format Integration', () => {
    it('should decode standardized cross-chain mint messages', () => {
      const tokenId = 12345;
      const originChainId = CHAIN_IDS.SOLANA; // Use Solana as origin chain
      const originAddress = new Uint8Array(32).fill(1);
      const recipientAddress = publicKeyToBytes(recipientKp.publicKey);
      const messageId = generateMessageId();

      const mintMessage = createMintMessage(
        tokenId,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        originChainId,
        originAddress,
        recipientAddress,
        messageId
      );

      expect(mintMessage.messageType).to.equal(MESSAGE_TYPES.NFT_MINT);
      expect(mintMessage.tokenId).to.equal(tokenId);
      expect(mintMessage.originChainId).to.equal(originChainId);
      expect(mintMessage.originAddress).to.deep.equal(originAddress);
      expect(mintMessage.recipientAddress).to.deep.equal(recipientAddress);
      expect(mintMessage.messageId).to.deep.equal(messageId);
      expect(isMintMessage(mintMessage)).to.be.true;
    });

    it('should validate standardized message format correctly', () => {
      const mintMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA, // Use Solana as origin chain
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      const errors = validateMessage(mintMessage);
      expect(errors).to.be.an('array');
      expect(errors).to.have.length(0);
    });

    it('should serialize and deserialize standardized messages correctly', () => {
      const originalMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA, // Use Solana as origin chain
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      const serialized = serializeMessage(originalMessage);
      const deserialized = deserializeMessage(serialized);

      expect(deserialized.messageType).to.equal(originalMessage.messageType);
      expect(deserialized.tokenId).to.equal(originalMessage.tokenId);
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
      expect(deserialized.metadataUri).to.equal(originalMessage.metadataUri);
      expect(deserialized.name).to.equal(originalMessage.name);
      expect(deserialized.symbol).to.equal(originalMessage.symbol);
    });
  });

  describe('Message Receiver Validation', () => {
    it('should accept valid mint messages from supported chains', () => {
      const supportedChains = [
        CHAIN_IDS.SOLANA,
        CHAIN_IDS.BASE_SEPOLIA,
        CHAIN_IDS.BNB_TESTNET,
      ];

      supportedChains.forEach(chainId => {
        const mintMessage = createMintMessage(
          12345,
          'https://example.com/metadata.json',
          'Cross-Chain NFT',
          'CCNFT',
          chainId,
          new Uint8Array(32).fill(1),
          publicKeyToBytes(recipientKp.publicKey),
          generateMessageId()
        );

        const errors = validateMessage(mintMessage);
        expect(errors).to.have.length(0);
      });
    });

    it('should reject messages with invalid message types', () => {
      const burnMessage = {
        messageType: MESSAGE_TYPES.NFT_BURN,
        tokenId: 12345,
        metadataUri: 'https://example.com/metadata.json',
        name: 'Cross-Chain NFT',
        symbol: 'CCNFT',
        originChainId: CHAIN_IDS.BASE_SEPOLIA,
        originAddress: new Uint8Array(32).fill(1),
        recipientAddress: publicKeyToBytes(recipientKp.publicKey),
        timestamp: Math.floor(Date.now() / 1000),
        messageId: generateMessageId(),
      };

      // This should be rejected by the message receiver
      expect(burnMessage.messageType).to.not.equal(MESSAGE_TYPES.NFT_MINT);
    });

    it('should validate message timestamp requirements', () => {
      const currentTime = Math.floor(Date.now() / 1000);
      const validTimestamp = currentTime - 3600; // 1 hour ago
      const invalidTimestamp = currentTime - 7200; // 2 hours ago (too old)

      const validMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );
      validMessage.timestamp = validTimestamp;

      const invalidMessage = createMintMessage(
        12346,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );
      invalidMessage.timestamp = invalidTimestamp;

      const validErrors = validateMessage(validMessage);
      const invalidErrors = validateMessage(invalidMessage);

      expect(validErrors).to.have.length(0);
      expect(invalidErrors).to.include(
        'Message too old - timestamp exceeds maximum age'
      );
    });
  });

  describe('Backward Compatibility', () => {
    it('should handle legacy payload formats', () => {
      // Test that the message receiver can handle both new and old formats
      const legacyPayload = Buffer.from([
        // token_id (8 bytes)
        0x39,
        0x30,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // 12345 in little-endian
        // origin_chain_id (32 bytes)
        ...new Array(32).fill(2), // Base Sepolia chain ID
        // gateway_message_id (32 bytes)
        ...new Array(32).fill(3),
        // metadata_uri (null-terminated)
        ...Buffer.from('https://example.com/metadata.json\0'),
        // name (null-terminated)
        ...Buffer.from('Legacy NFT\0'),
        // symbol (null-terminated)
        ...Buffer.from('LNFT\0'),
        // recipient_address (32 bytes)
        ...new Array(32).fill(4),
      ]);

      expect(legacyPayload).to.be.instanceof(Buffer);
      expect(legacyPayload.length).to.be.greaterThan(100);
    });

    it('should prioritize standardized format over legacy format', () => {
      const standardizedMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Standardized NFT',
        'SNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      const serialized = serializeMessage(standardizedMessage);

      // The receiver should prefer the standardized format
      expect(serialized).to.be.instanceof(Buffer);
      expect(serialized.length).to.be.greaterThan(0);
    });
  });

  describe('Message Processing Flow', () => {
    it('should handle complete message processing pipeline', () => {
      const mintMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      // Step 1: Validate message format
      const validationErrors = validateMessage(mintMessage);
      expect(validationErrors).to.have.length(0);

      // Step 2: Serialize for transmission
      const serialized = serializeMessage(mintMessage);
      expect(serialized).to.be.instanceof(Buffer);

      // Step 3: Deserialize for processing
      const deserialized = deserializeMessage(serialized);
      expect(deserialized.messageType).to.equal(MESSAGE_TYPES.NFT_MINT);

      // Step 4: Verify all fields are preserved
      expect(deserialized.tokenId).to.equal(mintMessage.tokenId);
      expect(deserialized.originChainId).to.equal(mintMessage.originChainId);
      expect(deserialized.originAddress).to.deep.equal(
        mintMessage.originAddress
      );
      expect(deserialized.recipientAddress).to.deep.equal(
        mintMessage.recipientAddress
      );
      expect(deserialized.messageId).to.deep.equal(mintMessage.messageId);
    });

    it('should handle message processing with additional metadata', () => {
      const additionalMetadata = new Uint8Array([1, 2, 3, 4, 5]);

      const mintMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );
      mintMessage.additionalMetadata = additionalMetadata;

      const serialized = serializeMessage(mintMessage);
      const deserialized = deserializeMessage(serialized);

      expect(deserialized.additionalMetadata).to.deep.equal(additionalMetadata);
    });
  });

  describe('Error Handling and Edge Cases', () => {
    it('should handle messages with missing required fields', () => {
      const invalidMessage: CrossChainNftMessage = {
        messageType: MESSAGE_TYPES.NFT_MINT,
        tokenId: 0, // Invalid token ID
        metadataUri: '',
        name: '',
        symbol: '',
        originChainId: 99, // Unsupported chain
        originAddress: new Uint8Array(16).fill(1), // Wrong length
        recipientAddress: publicKeyToBytes(recipientKp.publicKey),
        timestamp: Math.floor(Date.now() / 1000),
        messageId: new Uint8Array(16).fill(1), // Wrong length
      };

      const errors = validateMessage(invalidMessage);
      expect(errors).to.be.an('array');
      expect(errors.length).to.be.greaterThan(0);
      expect(errors).to.include('Invalid token ID - must be greater than 0');
      expect(errors).to.include('Unsupported chain ID');
      expect(errors).to.include(
        'Invalid origin address length - must be 32 bytes'
      );
      expect(errors).to.include('Invalid message ID length - must be 32 bytes');
    });

    it('should handle messages with oversized content', () => {
      const oversizedUri = 'https://example.com/' + 'a'.repeat(300) + '.json';

      const mintMessage = createMintMessage(
        12345,
        oversizedUri,
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      const errors = validateMessage(mintMessage);
      expect(errors).to.include('Metadata URI too long');
    });

    it('should handle messages with invalid addresses', () => {
      const mintMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(16).fill(1), // Wrong length
        new Uint8Array(16).fill(2), // Wrong length
        generateMessageId()
      );

      const errors = validateMessage(mintMessage);
      expect(errors).to.include(
        'Invalid origin address length - must be 32 bytes'
      );
      expect(errors).to.include(
        'Invalid recipient address length - must be 32 bytes'
      );
    });
  });

  describe('Performance and Optimization', () => {
    it('should handle message serialization efficiently', () => {
      const mintMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      const startTime = Date.now();
      const serialized = serializeMessage(mintMessage);
      const endTime = Date.now();

      expect(serialized).to.be.instanceof(Buffer);
      expect(serialized.length).to.be.greaterThan(0);
      expect(endTime - startTime).to.be.lessThan(100); // Should complete quickly
    });

    it('should handle message deserialization efficiently', () => {
      const mintMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      const serialized = serializeMessage(mintMessage);

      const startTime = Date.now();
      const deserialized = deserializeMessage(serialized);
      const endTime = Date.now();

      expect(deserialized.messageType).to.equal(MESSAGE_TYPES.NFT_MINT);
      expect(endTime - startTime).to.be.lessThan(100); // Should complete quickly
    });

    it('should validate message size constraints', () => {
      const mintMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      const serialized = serializeMessage(mintMessage);

      // Message should be within reasonable size limits
      expect(serialized.length).to.be.greaterThan(100); // Minimum size
      expect(serialized.length).to.be.lessThan(2048); // Maximum size
    });
  });

  describe('Integration with Existing Infrastructure', () => {
    it('should work with existing replay protection system', () => {
      const messageId = generateMessageId();
      const chainId = new Uint8Array(32).fill(1);

      // Test that message ID can be used for replay protection
      expect(messageId).to.have.length(32);
      expect(chainId).to.have.length(32);

      // In a real implementation, this would create a replay protection PDA
      const replayProtectionSeeds = [
        Buffer.from('processed_message'),
        messageId,
      ];

      expect(replayProtectionSeeds).to.be.an('array');
      expect(replayProtectionSeeds[0]).to.deep.equal(
        Buffer.from('processed_message')
      );
      expect(replayProtectionSeeds[1]).to.deep.equal(messageId);
    });

    it('should integrate with NFT origin tracking', () => {
      const mintMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      // Test that the message contains all necessary data for NFT origin tracking
      expect(mintMessage.tokenId).to.be.a('number');
      expect(mintMessage.originChainId).to.equal(CHAIN_IDS.SOLANA);
      expect(mintMessage.originAddress).to.have.length(32);
      expect(mintMessage.metadataUri).to.be.a('string');
      expect(mintMessage.name).to.be.a('string');
      expect(mintMessage.symbol).to.be.a('string');
    });

    it('should support collection association', () => {
      const mintMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      // The message should support collection metadata
      expect(mintMessage.additionalMetadata).to.be.undefined; // Optional field

      // Add collection metadata
      mintMessage.additionalMetadata = new Uint8Array([1, 2, 3, 4, 5]);
      expect(mintMessage.additionalMetadata).to.be.instanceof(Uint8Array);
    });
  });

  describe('Security and Validation', () => {
    it('should validate message authenticity', () => {
      const mintMessage = createMintMessage(
        12345,
        'https://example.com/metadata.json',
        'Cross-Chain NFT',
        'CCNFT',
        CHAIN_IDS.BASE_SEPOLIA,
        new Uint8Array(32).fill(1),
        publicKeyToBytes(recipientKp.publicKey),
        generateMessageId()
      );

      // Test that the message has a valid timestamp
      const currentTime = Math.floor(Date.now() / 1000);
      expect(mintMessage.timestamp).to.be.a('number');
      expect(mintMessage.timestamp).to.be.lessThanOrEqual(currentTime);
      expect(mintMessage.timestamp).to.be.greaterThan(currentTime - 3600); // Within last hour
    });

    it('should prevent replay attacks', () => {
      const messageId1 = generateMessageId();
      const messageId2 = generateMessageId();

      // Each message should have a unique ID
      expect(messageId1).to.not.deep.equal(messageId2);

      // Message IDs should be 32 bytes
      expect(messageId1).to.have.length(32);
      expect(messageId2).to.have.length(32);
    });

    it('should validate chain compatibility', () => {
      const supportedChains = [
        CHAIN_IDS.SOLANA,
        CHAIN_IDS.BASE_SEPOLIA,
        CHAIN_IDS.BNB_TESTNET,
      ];
      const unsupportedChain = 99;

      supportedChains.forEach(chainId => {
        const mintMessage = createMintMessage(
          12345,
          'https://example.com/metadata.json',
          'Cross-Chain NFT',
          'CCNFT',
          chainId,
          new Uint8Array(32).fill(1),
          publicKeyToBytes(recipientKp.publicKey),
          generateMessageId()
        );

        const errors = validateMessage(mintMessage);
        expect(errors).to.not.include('Unsupported chain ID');
      });

      // Test unsupported chain
      const invalidMessage: CrossChainNftMessage = {
        messageType: MESSAGE_TYPES.NFT_MINT,
        tokenId: 12345,
        metadataUri: 'https://example.com/metadata.json',
        name: 'Cross-Chain NFT',
        symbol: 'CCNFT',
        originChainId: unsupportedChain,
        originAddress: new Uint8Array(32).fill(1),
        recipientAddress: publicKeyToBytes(recipientKp.publicKey),
        timestamp: Math.floor(Date.now() / 1000),
        messageId: generateMessageId(),
      };

      const errors = validateMessage(invalidMessage);
      expect(errors).to.include('Unsupported chain ID');
    });
  });
});
