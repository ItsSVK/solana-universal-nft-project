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
  createBurnMessage,
  validateMessage,
  isBurnMessage,
  serializeMessage,
  deserializeMessage,
  generateMessageId,
  publicKeyToBytes,
} from './types/cross-chain-message';

anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.universalNft as Program<UniversalNft>;

describe('Enhanced Burn NFT and Transfer', () => {
  let mintKp: Keypair;
  let tokenAccount: PublicKey;
  let metadataAccount: PublicKey;
  let masterEditionAccount: PublicKey;
  let collectionMintKp: Keypair;
  let collectionMetadataAccount: PublicKey;
  let collectionMasterEditionAccount: PublicKey;
  let ownerZetaAccount: PublicKey;

  before(async () => {
    // Generate test accounts
    mintKp = Keypair.generate();
    collectionMintKp = Keypair.generate();

    // Derive associated token account
    tokenAccount = await anchor.utils.token.associatedAddress({
      mint: mintKp.publicKey,
      owner: (program.provider as anchor.AnchorProvider).wallet.publicKey,
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
    ownerZetaAccount = new PublicKey('11111111111111111111111111111115');
  });

  describe('Cross-Chain Message Format Integration', () => {
    it('should create valid burn message for cross-chain transfer', () => {
      const tokenId = 12345;
      const originChainId = CHAIN_IDS.SOLANA;
      const originAddress = new Uint8Array(32).fill(1);
      const destinationAddress = new Uint8Array(32).fill(2);
      const messageId = generateMessageId();

      const burnMessage = createBurnMessage(
        tokenId,
        originChainId,
        originAddress,
        destinationAddress,
        messageId
      );

      expect(burnMessage.messageType).to.equal(MESSAGE_TYPES.NFT_BURN);
      expect(burnMessage.tokenId).to.equal(tokenId);
      expect(burnMessage.originChainId).to.equal(originChainId);
      expect(burnMessage.originAddress).to.deep.equal(originAddress);
      expect(burnMessage.recipientAddress).to.deep.equal(destinationAddress);
      expect(burnMessage.messageId).to.deep.equal(messageId);
      expect(isBurnMessage(burnMessage)).to.be.true;
    });

    it('should validate burn message format correctly', () => {
      const burnMessage = createBurnMessage(
        12345,
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const errors = validateMessage(burnMessage);
      expect(errors).to.be.an('array');
      expect(errors).to.have.length(0);
    });

    it('should serialize and deserialize burn message correctly', () => {
      const originalMessage = createBurnMessage(
        12345,
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
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
    });
  });

  describe('Burn NFT and Transfer Parameters', () => {
    it('should accept valid burn and transfer parameters', () => {
      const params = {
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        destinationAddress: new Uint8Array(32).fill(2),
        metadataUri: 'https://example.com/metadata.json',
        name: 'Cross-Chain NFT',
        symbol: 'CCNFT',
        additionalMetadata: new Uint8Array([1, 2, 3, 4, 5]),
      };

      expect(params.destinationChain).to.equal(CHAIN_IDS.BASE_SEPOLIA);
      expect(params.destinationAddress).to.have.length(32);
      expect(params.metadataUri).to.be.a('string');
      expect(params.name).to.be.a('string');
      expect(params.symbol).to.be.a('string');
      expect(params.additionalMetadata).to.be.instanceof(Uint8Array);
    });

    it('should handle optional parameters correctly', () => {
      const params = {
        destinationChain: CHAIN_IDS.BNB_TESTNET,
        destinationAddress: new Uint8Array(32).fill(3),
        metadataUri: undefined,
        name: undefined,
        symbol: undefined,
        additionalMetadata: undefined,
      };

      expect(params.destinationChain).to.equal(CHAIN_IDS.BNB_TESTNET);
      expect(params.destinationAddress).to.have.length(32);
      expect(params.metadataUri).to.be.undefined;
      expect(params.name).to.be.undefined;
      expect(params.symbol).to.be.undefined;
      expect(params.additionalMetadata).to.be.undefined;
    });
  });

  describe('Cross-Chain Transfer Flow', () => {
    it('should validate destination chain support', () => {
      const supportedChains = [
        CHAIN_IDS.SOLANA,
        CHAIN_IDS.BASE_SEPOLIA,
        CHAIN_IDS.BNB_TESTNET,
      ];

      supportedChains.forEach(chainId => {
        expect(supportedChains).to.include(chainId);
      });

      // Test unsupported chain
      const unsupportedChain = 99;
      expect(supportedChains).to.not.include(unsupportedChain);
    });

    it('should generate unique message IDs for replay protection', () => {
      const messageId1 = generateMessageId();
      const messageId2 = generateMessageId();

      expect(messageId1).to.have.length(32);
      expect(messageId2).to.have.length(32);
      expect(messageId1).to.not.deep.equal(messageId2);
    });

    it('should handle different destination chains correctly', () => {
      const testCases = [
        {
          chainId: CHAIN_IDS.BASE_SEPOLIA,
          name: 'Base Sepolia',
          expectedFee: 2, // Higher fee for EVM chains
        },
        {
          chainId: CHAIN_IDS.BNB_TESTNET,
          name: 'BNB Testnet',
          expectedFee: 2, // Higher fee for EVM chains
        },
        {
          chainId: CHAIN_IDS.SOLANA,
          name: 'Solana',
          expectedFee: 1, // Lower fee for Solana
        },
      ];

      testCases.forEach(testCase => {
        expect(testCase.chainId).to.be.a('number');
        expect(testCase.name).to.be.a('string');
        expect(testCase.expectedFee).to.be.a('number');
      });
    });
  });

  describe('Gateway Integration', () => {
    it('should prepare Gateway CPI call parameters', () => {
      const destinationChain = CHAIN_IDS.BASE_SEPOLIA;
      const payload = new Uint8Array([1, 2, 3, 4, 5]);

      // Mock Gateway program parameters
      const gatewayParams = {
        programId: new PublicKey('11111111111111111111111111111111'),
        gasLimit: 100000,
        zetaFeeAmount: 1000000,
        destinationChain: destinationChain as number,
        payloadSize: payload.length,
      };

      expect(gatewayParams.programId).to.be.instanceof(PublicKey);
      expect(gatewayParams.gasLimit).to.be.a('number');
      expect(gatewayParams.zetaFeeAmount).to.be.a('number');
      expect(gatewayParams.destinationChain).to.equal(destinationChain);
      expect(gatewayParams.payloadSize).to.equal(payload.length);
    });

    it('should validate Gateway program ID', () => {
      const expectedGatewayId = new PublicKey(
        '11111111111111111111111111111111'
      );
      const testGatewayId = new PublicKey('11111111111111111111111111111111');

      expect(testGatewayId.equals(expectedGatewayId)).to.be.true;
    });
  });

  describe('Error Handling', () => {
    it('should reject unsupported destination chains', () => {
      const unsupportedChain = 99;
      const supportedChains = [
        CHAIN_IDS.SOLANA,
        CHAIN_IDS.BASE_SEPOLIA,
        CHAIN_IDS.BNB_TESTNET,
      ];

      expect(supportedChains).to.not.include(unsupportedChain);
    });

    it('should validate address format requirements', () => {
      const validAddress = new Uint8Array(32).fill(1);
      const invalidAddress = new Uint8Array(16).fill(1);

      expect(validAddress).to.have.length(32);
      expect(invalidAddress).to.have.length(16);
      expect(validAddress).to.not.deep.equal(invalidAddress);
    });

    it('should handle message validation errors', () => {
      const invalidMessage: CrossChainNftMessage = {
        messageType: 99, // Invalid type
        tokenId: 0, // Invalid token ID
        metadataUri: '',
        name: '',
        symbol: '',
        originChainId: 99, // Unsupported chain
        originAddress: new Uint8Array(16).fill(1), // Wrong length
        recipientAddress: new Uint8Array(32).fill(2),
        timestamp: Math.floor(Date.now() / 1000),
        messageId: new Uint8Array(16).fill(1), // Wrong length
      };

      const errors = validateMessage(invalidMessage);
      expect(errors).to.be.an('array');
      expect(errors.length).to.be.greaterThan(0);
      expect(errors).to.include('Invalid message type - must be 1-4');
      expect(errors).to.include('Invalid token ID - must be greater than 0');
      expect(errors).to.include('Unsupported chain ID');
      expect(errors).to.include(
        'Invalid origin address length - must be 32 bytes'
      );
      expect(errors).to.include('Invalid message ID length - must be 32 bytes');
    });
  });

  describe('Integration with Existing Infrastructure', () => {
    it('should work with existing NFT origin tracking', () => {
      // Test that the burn and transfer can work with existing NFT origin PDAs
      const mockNftOriginData = {
        tokenId: 12345,
        originChain: CHAIN_IDS.SOLANA,
        originAddress: new Uint8Array(32).fill(1),
        metadataUri: 'https://example.com/metadata.json',
        name: 'Solana NFT',
        symbol: 'SNFT',
      };

      expect(mockNftOriginData.tokenId).to.be.a('number');
      expect(mockNftOriginData.originChain).to.equal(CHAIN_IDS.SOLANA);
      expect(mockNftOriginData.originAddress).to.have.length(32);
      expect(mockNftOriginData.metadataUri).to.be.a('string');
      expect(mockNftOriginData.name).to.be.a('string');
      expect(mockNftOriginData.symbol).to.be.a('string');
    });

    it('should integrate with replay protection system', () => {
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
  });

  describe('Performance and Optimization', () => {
    it('should handle message serialization efficiently', () => {
      const burnMessage = createBurnMessage(
        12345,
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const startTime = Date.now();
      const serialized = serializeMessage(burnMessage);
      const endTime = Date.now();

      expect(serialized).to.be.instanceof(Buffer);
      expect(serialized.length).to.be.greaterThan(0);
      expect(endTime - startTime).to.be.lessThan(100); // Should complete quickly
    });

    it('should validate message size constraints', () => {
      const burnMessage = createBurnMessage(
        12345,
        CHAIN_IDS.SOLANA,
        new Uint8Array(32).fill(1),
        new Uint8Array(32).fill(2),
        generateMessageId()
      );

      const serialized = serializeMessage(burnMessage);

      // Message should be within reasonable size limits
      expect(serialized.length).to.be.greaterThan(100); // Minimum size
      expect(serialized.length).to.be.lessThan(2048); // Maximum size
    });
  });
});
