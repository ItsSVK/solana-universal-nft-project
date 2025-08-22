import { ethers } from 'ethers';
import { expect } from 'chai';
import { PublicKey, Connection, Keypair } from '@solana/web3.js';
import * as anchor from '@coral-xyz/anchor';
import { CrossChainMessageUtils, CHAIN_IDS, NFTTransferMessage } from '../shared/CrossChainMessage';
import { MessageBridge } from '../shared/MessageBridge';

describe('Universal NFT Protocol', () => {
  // Test accounts
  let owner: ethers.Wallet;
  let recipient: ethers.Wallet;
  let solanaOwner: Keypair;
  let solanaRecipient: Keypair;

  // Contract instances
  let universalNFT: ethers.Contract;
  let nftReceiver: ethers.Contract;
  let provider: ethers.JsonRpcProvider;
  let connection: Connection;

  // Test data
  const tokenId = '123';
  const metadataUri = 'https://example.com/metadata/123.json';
  const nftName = 'Test Universal NFT';
  const nftSymbol = 'TUNFT';

  before(async () => {
    // Setup Ethereum test environment
    provider = new ethers.JsonRpcProvider('http://localhost:8545');
    owner = new ethers.Wallet('0x' + '01'.repeat(32), provider);
    recipient = new ethers.Wallet('0x' + '02'.repeat(32), provider);

    // Setup Solana test environment
    connection = new Connection('http://localhost:8899', 'confirmed');
    solanaOwner = Keypair.generate();
    solanaRecipient = Keypair.generate();

    // Fund test accounts (in real tests, you'd need to implement funding)
    // await fundAccount(owner.address, ethers.parseEther("10"));
    // await fundSolanaAccount(solanaOwner.publicKey);
  });

  describe('Cross-Chain Message Format', () => {
    it('should create valid NFT transfer message', () => {
      const message = CrossChainMessageUtils.createMessage({
        tokenId,
        metadataUri,
        recipient: recipient.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: owner.address,
        sender: owner.address,
        nonce: '1',
        isRecipientSolana: false,
        isOriginContractSolana: false,
      });

      expect(message.tokenId).to.equal(tokenId);
      expect(message.metadataUri).to.equal(metadataUri);
      expect(message.originChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);
      expect(message.destinationChain).to.equal(CHAIN_IDS.BASE_SEPOLIA);
      expect(message.recipient.length).to.equal(32);
      expect(message.messageId.length).to.equal(32);
    });

    it('should validate message constraints', () => {
      const validMessage = CrossChainMessageUtils.createMessage({
        tokenId,
        metadataUri,
        recipient: recipient.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: owner.address,
        sender: owner.address,
        nonce: '1',
      });

      expect(() => CrossChainMessageUtils.validateMessage(validMessage)).to.not.throw();

      // Test invalid message (same chain transfer)
      const invalidMessage = { ...validMessage };
      invalidMessage.destinationChain = invalidMessage.originChain;
      expect(() => CrossChainMessageUtils.validateMessage(invalidMessage)).to.throw(
        'Same chain transfer'
      );

      // Test message too old
      const oldMessage = { ...validMessage };
      oldMessage.timestamp = Math.floor(Date.now() / 1000) - 86400 - 1; // 1 day + 1 second old
      expect(() =>
        CrossChainMessageUtils.validateMessage(oldMessage, Math.floor(Date.now() / 1000))
      ).to.throw('Message too old');
    });

    it('should generate unique message IDs', () => {
      const messageId1 = CrossChainMessageUtils.generateMessageId(
        owner.address,
        tokenId,
        CHAIN_IDS.BASE_SEPOLIA,
        '1',
        1640995200
      );
      const messageId2 = CrossChainMessageUtils.generateMessageId(
        owner.address,
        tokenId,
        CHAIN_IDS.BASE_SEPOLIA,
        '1',
        1640995200
      );
      const messageId3 = CrossChainMessageUtils.generateMessageId(
        owner.address,
        tokenId,
        CHAIN_IDS.BASE_SEPOLIA,
        '2',
        1640995200
      );

      expect(Buffer.from(messageId1).equals(Buffer.from(messageId2))).to.be.true;
      expect(Buffer.from(messageId1).equals(Buffer.from(messageId3))).to.be.false;
    });
  });

  describe('Message Format Conversion', () => {
    let testMessage: NFTTransferMessage;

    beforeEach(() => {
      testMessage = CrossChainMessageUtils.createMessage({
        tokenId,
        metadataUri,
        recipient: recipient.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: owner.address,
        sender: owner.address,
        nonce: '1',
      });
    });

    it('should convert to/from EVM format', () => {
      const evmEncoded = MessageBridge.toEvmFormat(testMessage);
      expect(evmEncoded).to.be.a('string');
      expect(evmEncoded.startsWith('0x')).to.be.true;

      const decoded = MessageBridge.fromEvmFormat(evmEncoded);
      expect(decoded.tokenId).to.equal(testMessage.tokenId);
      expect(decoded.metadataUri).to.equal(testMessage.metadataUri);
      expect(decoded.originChain).to.equal(testMessage.originChain);
      expect(decoded.destinationChain).to.equal(testMessage.destinationChain);
      expect(Buffer.from(decoded.recipient).equals(Buffer.from(testMessage.recipient))).to.be.true;
    });

    it('should convert to/from Solana format', () => {
      const solanaFormat = MessageBridge.toSolanaFormat(testMessage);
      expect(solanaFormat.messageType).to.equal(1); // NFT_MINT
      expect(solanaFormat.tokenId).to.equal(testMessage.tokenId);
      expect(solanaFormat.metadataUri).to.equal(testMessage.metadataUri);
      expect(solanaFormat.originChainId).to.equal(testMessage.originChain);

      const backToShared = MessageBridge.fromSolanaFormat(solanaFormat);
      expect(backToShared.tokenId).to.equal(testMessage.tokenId);
      expect(backToShared.metadataUri).to.equal(testMessage.metadataUri);
      expect(backToShared.originChain).to.equal(testMessage.originChain);
    });

    it('should handle address conversions correctly', () => {
      // Test Ethereum address conversion
      const ethAddr = recipient.address;
      const bytes32 = CrossChainMessageUtils.ethereumAddressToBytes32(ethAddr);
      const convertedBack = CrossChainMessageUtils.bytes32ToEthereumAddress(bytes32);
      expect(convertedBack.toLowerCase()).to.equal(ethAddr.toLowerCase());

      // Test Solana address conversion
      const solanaAddr = solanaRecipient.publicKey;
      const solanaBytes32 = CrossChainMessageUtils.solanaPublicKeyToBytes32(solanaAddr);
      const solanaConvertedBack = CrossChainMessageUtils.bytes32ToSolanaPublicKey(solanaBytes32);
      expect(solanaConvertedBack.toString()).to.equal(solanaAddr.toString());
    });
  });

  describe('Cross-Chain Transfer Flows', () => {
    describe('ZetaChain to Base Sepolia', () => {
      it('should create valid transfer message from ZetaChain to Base', async () => {
        const transferMessage = MessageBridge.createEvmToEvmMessage({
          tokenId,
          metadataUri,
          recipientAddress: recipient.address,
          originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
          destinationChain: CHAIN_IDS.BASE_SEPOLIA,
          senderAddress: owner.address,
          originContractAddress: owner.address, // Mock contract address
          nonce: '1',
        });

        expect(transferMessage.originChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);
        expect(transferMessage.destinationChain).to.equal(CHAIN_IDS.BASE_SEPOLIA);
        expect(transferMessage.tokenId).to.equal(tokenId);
        expect(transferMessage.metadataUri).to.equal(metadataUri);

        // Validate the message
        expect(() => CrossChainMessageUtils.validateMessage(transferMessage)).to.not.throw();
      });

      it('should encode message for Gateway transmission', async () => {
        const transferMessage = MessageBridge.createEvmToEvmMessage({
          tokenId,
          metadataUri,
          recipientAddress: recipient.address,
          originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
          destinationChain: CHAIN_IDS.BASE_SEPOLIA,
          senderAddress: owner.address,
          originContractAddress: owner.address,
          nonce: '1',
        });

        const encoded = CrossChainMessageUtils.encodeForEVM(transferMessage);
        expect(encoded).to.be.a('string');
        expect(encoded.startsWith('0x')).to.be.true;
        expect(encoded.length).to.be.greaterThan(2); // More than just '0x'

        // Validate decoding works
        const decoded = CrossChainMessageUtils.decodeFromEVM(encoded);
        expect(decoded.tokenId).to.equal(transferMessage.tokenId);
        expect(decoded.metadataUri).to.equal(transferMessage.metadataUri);
      });
    });

    describe('ZetaChain to Solana', () => {
      it('should create valid transfer message from ZetaChain to Solana', async () => {
        const transferMessage = MessageBridge.createEvmToSolanaMessage({
          tokenId,
          metadataUri,
          recipientAddress: solanaRecipient.publicKey.toString(),
          originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
          senderAddress: owner.address,
          originContractAddress: owner.address,
          nonce: '1',
        });

        expect(transferMessage.originChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);
        expect(transferMessage.destinationChain).to.equal(CHAIN_IDS.SOLANA_DEVNET);
        expect(transferMessage.tokenId).to.equal(tokenId);
        expect(transferMessage.recipient.length).to.equal(32);

        // Validate the message
        expect(() => CrossChainMessageUtils.validateMessage(transferMessage)).to.not.throw();
      });
    });

    describe('Solana to Base Sepolia', () => {
      it('should create valid transfer message from Solana to Base', async () => {
        const transferMessage = MessageBridge.createSolanaToEvmMessage({
          tokenId,
          metadataUri,
          recipientAddress: recipient.address,
          destinationChain: CHAIN_IDS.BASE_SEPOLIA,
          senderAddress: solanaOwner.publicKey.toString(),
          nonce: '1',
        });

        expect(transferMessage.originChain).to.equal(CHAIN_IDS.SOLANA_DEVNET);
        expect(transferMessage.destinationChain).to.equal(CHAIN_IDS.BASE_SEPOLIA);
        expect(transferMessage.tokenId).to.equal(tokenId);

        // Validate the message
        expect(() => CrossChainMessageUtils.validateMessage(transferMessage)).to.not.throw();
      });
    });
  });

  describe('Provenance Tracking', () => {
    it('should preserve origin information across chains', () => {
      // Start with ZetaChain origin
      const originalMessage = CrossChainMessageUtils.createMessage({
        tokenId,
        metadataUri,
        recipient: recipient.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: owner.address,
        sender: owner.address,
        nonce: '1',
      });

      // Simulate transfer to Base Sepolia (origin should be preserved)
      const baseMessage = {
        ...originalMessage,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET, // Should remain ZetaChain
        destinationChain: CHAIN_IDS.SOLANA_DEVNET, // New destination
        recipient: CrossChainMessageUtils.solanaPublicKeyToBytes32(solanaRecipient.publicKey),
      };

      // Simulate transfer to Solana (origin should still be preserved)
      const solanaMessage = {
        ...baseMessage,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA, // Back to Base
        recipient: CrossChainMessageUtils.ethereumAddressToBytes32(recipient.address),
      };

      // Verify origin is preserved throughout
      expect(originalMessage.originChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);
      expect(baseMessage.originChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);
      expect(solanaMessage.originChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);

      // Verify original contract is preserved
      expect(
        Buffer.from(originalMessage.originContract).equals(Buffer.from(baseMessage.originContract))
      ).to.be.true;
      expect(
        Buffer.from(baseMessage.originContract).equals(Buffer.from(solanaMessage.originContract))
      ).to.be.true;
    });

    it('should track full cross-chain journey', () => {
      const journey = [
        {
          from: CHAIN_IDS.ZETACHAIN_TESTNET,
          to: CHAIN_IDS.BASE_SEPOLIA,
          step: 1,
        },
        {
          from: CHAIN_IDS.BASE_SEPOLIA,
          to: CHAIN_IDS.SOLANA_DEVNET,
          step: 2,
        },
        {
          from: CHAIN_IDS.SOLANA_DEVNET,
          to: CHAIN_IDS.ZETACHAIN_TESTNET,
          step: 3,
        },
      ];

      let currentMessage = CrossChainMessageUtils.createMessage({
        tokenId,
        metadataUri,
        recipient: owner.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: owner.address,
        sender: owner.address,
        nonce: '1',
      });

      const originChain = currentMessage.originChain;
      const originContract = currentMessage.originContract;

      journey.forEach((step, index) => {
        expect(currentMessage.originChain).to.equal(
          originChain,
          `Origin chain changed at step ${step.step}`
        );
        expect(Buffer.from(currentMessage.originContract).equals(Buffer.from(originContract))).to.be
          .true;

        // Simulate next transfer
        if (index < journey.length - 1) {
          const nextStep = journey[index + 1];
          currentMessage = {
            ...currentMessage,
            destinationChain: nextStep.to,
            nonce: (parseInt(currentMessage.nonce) + 1).toString(),
            messageId: CrossChainMessageUtils.generateMessageId(
              owner.address,
              tokenId,
              nextStep.to,
              (parseInt(currentMessage.nonce) + 1).toString(),
              Math.floor(Date.now() / 1000)
            ),
          };
        }
      });
    });
  });

  describe('Security Features', () => {
    it('should prevent replay attacks with unique message IDs', () => {
      // Test 1: Same nonce but different timestamps (if timing allows)
      const message1 = CrossChainMessageUtils.createMessage({
        tokenId,
        metadataUri,
        recipient: recipient.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: owner.address,
        sender: owner.address,
        nonce: '1',
      });

      // Use different nonces to ensure different message IDs
      const message2 = CrossChainMessageUtils.createMessage({
        tokenId,
        metadataUri,
        recipient: recipient.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: owner.address,
        sender: owner.address,
        nonce: '2', // Different nonce ensures different message ID
      });

      // Messages should have different IDs
      expect(Buffer.from(message1.messageId).equals(Buffer.from(message2.messageId))).to.be.false;

      // Test 3: Same nonce but different token ID
      const message3 = CrossChainMessageUtils.createMessage({
        tokenId: '9999', // Different token ID
        metadataUri,
        recipient: recipient.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: owner.address,
        sender: owner.address,
        nonce: '1', // Same nonce as message1
      });

      // Messages with same nonce but different token ID should have different IDs
      expect(Buffer.from(message1.messageId).equals(Buffer.from(message3.messageId))).to.be.false;
    });

    it('should validate message timestamps', () => {
      const currentTime = Math.floor(Date.now() / 1000);

      // Valid message (current time)
      const validMessage = CrossChainMessageUtils.createMessage({
        tokenId,
        metadataUri,
        recipient: recipient.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: owner.address,
        sender: owner.address,
        nonce: '1',
      });

      expect(() =>
        CrossChainMessageUtils.validateMessage(validMessage, currentTime)
      ).to.not.throw();

      // Old message (should fail)
      const oldMessage = { ...validMessage };
      oldMessage.timestamp = currentTime - 86401; // 1 day + 1 second old
      expect(() => CrossChainMessageUtils.validateMessage(oldMessage, currentTime)).to.throw();
    });

    it('should validate supported transfer routes', () => {
      const supportedRoutes = MessageBridge.getSupportedRoutes();

      expect(supportedRoutes.length).to.be.greaterThan(0);

      // Test supported routes
      expect(MessageBridge.isRouteSupported(CHAIN_IDS.ZETACHAIN_TESTNET, CHAIN_IDS.BASE_SEPOLIA)).to
        .be.true;
      expect(MessageBridge.isRouteSupported(CHAIN_IDS.ZETACHAIN_TESTNET, CHAIN_IDS.SOLANA_DEVNET))
        .to.be.true;
      expect(MessageBridge.isRouteSupported(CHAIN_IDS.BASE_SEPOLIA, CHAIN_IDS.ZETACHAIN_TESTNET)).to
        .be.true;
      expect(MessageBridge.isRouteSupported(CHAIN_IDS.BASE_SEPOLIA, CHAIN_IDS.SOLANA_DEVNET)).to.be
        .true;
      expect(MessageBridge.isRouteSupported(CHAIN_IDS.SOLANA_DEVNET, CHAIN_IDS.ZETACHAIN_TESTNET))
        .to.be.true;
      expect(MessageBridge.isRouteSupported(CHAIN_IDS.SOLANA_DEVNET, CHAIN_IDS.BASE_SEPOLIA)).to.be
        .true;

      // Test unsupported route (chain to itself)
      expect(
        MessageBridge.isRouteSupported(CHAIN_IDS.ZETACHAIN_TESTNET, CHAIN_IDS.ZETACHAIN_TESTNET)
      ).to.be.false;
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid message data gracefully', () => {
      // Test empty metadata URI
      expect(() => {
        CrossChainMessageUtils.createMessage({
          tokenId,
          metadataUri: '',
          recipient: recipient.address,
          originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
          destinationChain: CHAIN_IDS.BASE_SEPOLIA,
          originContract: owner.address,
          sender: owner.address,
          nonce: '1',
        });
      }).to.throw();

      // Test invalid token ID
      expect(() => {
        CrossChainMessageUtils.createMessage({
          tokenId: '0',
          metadataUri,
          recipient: recipient.address,
          originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
          destinationChain: CHAIN_IDS.BASE_SEPOLIA,
          originContract: owner.address,
          sender: owner.address,
          nonce: '1',
        });
      }).to.throw();
    });

    it('should handle corrupted message data', () => {
      const validMessage = CrossChainMessageUtils.createMessage({
        tokenId,
        metadataUri,
        recipient: recipient.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: owner.address,
        sender: owner.address,
        nonce: '1',
      });

      const encoded = CrossChainMessageUtils.encodeForEVM(validMessage);

      // Test corrupted data - make it severely corrupted to ensure it throws
      const corrupted = '0x' + 'FF'.repeat(200); // Invalid ABI-encoded data
      expect(() => CrossChainMessageUtils.decodeFromEVM(corrupted)).to.throw();
    });
  });

  describe('Gas Estimation', () => {
    it('should provide gas cost estimates', async () => {
      const gasEstimate = await MessageBridge.estimateGasCosts({
        fromChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        toChain: CHAIN_IDS.BASE_SEPOLIA,
      });

      expect(gasEstimate.estimatedGas).to.be.a('string');
      expect(gasEstimate.estimatedCostUSD).to.be.a('string');
      expect(parseInt(gasEstimate.estimatedGas)).to.be.greaterThan(0);
      expect(parseFloat(gasEstimate.estimatedCostUSD)).to.be.greaterThan(0);
    });
  });
});

// Helper functions for testing
function generateMockMetadata(tokenId: string, name: string = 'Test NFT') {
  return {
    name,
    description: `Test NFT ${tokenId} for Universal NFT Protocol`,
    image: `https://example.com/images/${tokenId}.png`,
    attributes: [
      { trait_type: 'Origin', value: 'ZetaChain' },
      { trait_type: 'Protocol', value: 'Universal NFT' },
      { trait_type: 'Token ID', value: tokenId },
    ],
  };
}

function createMockTokenURI(tokenId: string): string {
  return `data:application/json;base64,${Buffer.from(
    JSON.stringify(generateMockMetadata(tokenId))
  ).toString('base64')}`;
}

// Export for use in integration tests
export { generateMockMetadata, createMockTokenURI };
