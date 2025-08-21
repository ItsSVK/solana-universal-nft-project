import { expect } from 'chai';
import { ethers } from 'hardhat';
import { UniversalNFTReceiver } from '../typechain-types';
import { CrossChainMessageUtils, CHAIN_IDS } from '../shared/CrossChainMessage';

describe('UniversalNFTReceiver (Base Sepolia)', () => {
  let nftReceiver: UniversalNFTReceiver;
  let owner: any;
  let user1: any;
  let user2: any;
  let mockGateway: any;
  let mockZetaContract: string;

  const TEST_METADATA_URI = 'https://example.com/metadata/123.json';

  before(async () => {
    [owner, user1, user2, mockGateway] = await ethers.getSigners();
    mockZetaContract = user2.address; // Mock ZetaChain contract address
  });

  beforeEach(async () => {
    const UniversalNFTReceiverFactory = await ethers.getContractFactory('UniversalNFTReceiver');
    nftReceiver = await UniversalNFTReceiverFactory.deploy(
      mockGateway.address,
      owner.address,
      mockZetaContract
    );
    await nftReceiver.waitForDeployment();
  });

  describe('Contract Deployment', () => {
    it('should deploy with correct parameters', async () => {
      expect(await nftReceiver.owner()).to.equal(owner.address);
      expect(await nftReceiver.getZetaChainContract()).to.equal(mockZetaContract);
      expect(await nftReceiver.name()).to.equal('Universal NFT Receiver');
      expect(await nftReceiver.symbol()).to.equal('UNFTR');
    });

    it('should start with token counter at 0', async () => {
      expect(await nftReceiver.getCurrentTokenId()).to.equal(0);
    });
  });

  describe('Local NFT Minting', () => {
    it('should mint NFT locally on Base Sepolia', async () => {
      const tx = await nftReceiver.mint(user1.address, TEST_METADATA_URI);
      const receipt = await tx.wait();

      expect(await nftReceiver.ownerOf(0)).to.equal(user1.address);
      expect(await nftReceiver.tokenURI(0)).to.equal(TEST_METADATA_URI);
      expect(await nftReceiver.getCurrentTokenId()).to.equal(1);

      // Check origin information
      const tokenOrigin = await nftReceiver.getTokenOrigin(0);
      expect(tokenOrigin.originChain).to.equal(CHAIN_IDS.BASE_SEPOLIA);
      expect(tokenOrigin.metadataUri).to.equal(TEST_METADATA_URI);
    });

    it('should reject minting with empty metadata URI', async () => {
      await expect(
        nftReceiver.mint(user1.address, '')
      ).to.be.revertedWith('Invalid metadata URI');
    });

    it('should reject minting with metadata URI too long', async () => {
      const longUri = 'x'.repeat(501);
      await expect(
        nftReceiver.mint(user1.address, longUri)
      ).to.be.revertedWith('Invalid metadata URI');
    });
  });

  describe('Cross-Chain Message Receiving', () => {
    it('should receive and process cross-chain NFT message', async () => {
      // Create a cross-chain message
      const message = CrossChainMessageUtils.createMessage({
        tokenId: '123',
        metadataUri: TEST_METADATA_URI,
        recipient: user1.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: mockZetaContract,
        sender: user2.address,
        nonce: '1'
      });

      const encodedMessage = CrossChainMessageUtils.encodeForEVM(message);
      
      // Mock message context
      const mockContext = {
        sender: mockZetaContract,
        origin: ethers.ZeroAddress,
        chainId: CHAIN_IDS.ZETACHAIN_TESTNET
      };

      // Simulate call from gateway
      const tx = await nftReceiver.connect(mockGateway).onCall(
        mockContext,
        encodedMessage
      );
      
      const receipt = await tx.wait();

      // Verify NFT was minted
      expect(await nftReceiver.ownerOf(0)).to.equal(user1.address);
      expect(await nftReceiver.tokenURI(0)).to.equal(TEST_METADATA_URI);

      // Verify origin preservation
      const tokenOrigin = await nftReceiver.getTokenOrigin(0);
      expect(tokenOrigin.originChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);
      expect(tokenOrigin.metadataUri).to.equal(TEST_METADATA_URI);

      // Verify message is marked as processed
      expect(await nftReceiver.isMessageProcessed(message.messageId)).to.be.true;
    });

    it('should reject calls from non-gateway addresses', async () => {
      const message = CrossChainMessageUtils.createMessage({
        tokenId: '123',
        metadataUri: TEST_METADATA_URI,
        recipient: user1.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: mockZetaContract,
        sender: user2.address,
        nonce: '1'
      });

      const encodedMessage = CrossChainMessageUtils.encodeForEVM(message);
      const mockContext = {
        sender: mockZetaContract,
        origin: ethers.ZeroAddress,
        chainId: CHAIN_IDS.ZETACHAIN_TESTNET
      };

      await expect(
        nftReceiver.connect(user1).onCall(mockContext, encodedMessage)
      ).to.be.revertedWithCustomError(nftReceiver, 'Unauthorized');
    });

    it('should prevent replay attacks', async () => {
      const message = CrossChainMessageUtils.createMessage({
        tokenId: '124',
        metadataUri: TEST_METADATA_URI,
        recipient: user1.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        originContract: mockZetaContract,
        sender: user2.address,
        nonce: '2'
      });

      const encodedMessage = CrossChainMessageUtils.encodeForEVM(message);
      const mockContext = {
        sender: mockZetaContract,
        origin: ethers.ZeroAddress,
        chainId: CHAIN_IDS.ZETACHAIN_TESTNET
      };

      // First call should succeed
      await nftReceiver.connect(mockGateway).onCall(mockContext, encodedMessage);

      // Second call should fail
      await expect(
        nftReceiver.connect(mockGateway).onCall(mockContext, encodedMessage)
      ).to.be.revertedWith('Message already processed');
    });
  });

  describe('Cross-Chain Message Construction and Sending', () => {
    beforeEach(async () => {
      // Mint an NFT first
      await nftReceiver.mint(user1.address, TEST_METADATA_URI);
    });

    it('should validate destination chains', async () => {
      await expect(
        nftReceiver.connect(user1).burnAndTransfer(
          0,
          CHAIN_IDS.BASE_SEPOLIA, // Same chain transfer
          ethers.hexlify(ethers.randomBytes(32))
        )
      ).to.be.revertedWith('Invalid destination chain');
    });

    it('should only allow token owner to burn and transfer', async () => {
      await expect(
        nftReceiver.connect(user2).burnAndTransfer(
          0,
          CHAIN_IDS.ZETACHAIN_TESTNET,
          ethers.hexlify(ethers.randomBytes(32))
        )
      ).to.be.revertedWith('Not token owner');
    });

    it('should provide convenience function for ZetaChain transfers', async () => {
      const recipient = user2.address;
      
      // This would normally send a cross-chain message, but we can't test that without a real gateway
      // Instead, we verify the function calls complete without error
      const tx = await nftReceiver.connect(user1).burnAndTransferToZetaChain(0, recipient);
      const receipt = await tx.wait();

      // Verify NFT was burned
      await expect(nftReceiver.ownerOf(0)).to.be.revertedWith('ERC721: invalid token ID');
      
      // Verify user nonce was incremented
      expect(await nftReceiver.getUserNonce(user1.address)).to.equal(1);
    });

    it('should provide convenience function for Solana transfers', async () => {
      const recipientPubkey = ethers.hexlify(ethers.randomBytes(32));
      
      const tx = await nftReceiver.connect(user1).burnAndTransferToSolana(0, recipientPubkey);
      const receipt = await tx.wait();

      // Verify NFT was burned
      await expect(nftReceiver.ownerOf(0)).to.be.revertedWith('ERC721: invalid token ID');
      
      // Verify user nonce was incremented
      expect(await nftReceiver.getUserNonce(user1.address)).to.equal(1);
    });
  });

  describe('Admin Functions', () => {
    it('should allow owner to update ZetaChain contract address', async () => {
      const newZetaContract = user1.address;
      await nftReceiver.setZetaChainContract(newZetaContract);
      expect(await nftReceiver.getZetaChainContract()).to.equal(newZetaContract);
    });

    it('should not allow non-owner to update ZetaChain contract', async () => {
      await expect(
        nftReceiver.connect(user1).setZetaChainContract(user1.address)
      ).to.be.revertedWithCustomError(nftReceiver, 'OwnableUnauthorizedAccount');
    });

    it('should allow owner to pause and unpause', async () => {
      await nftReceiver.pause();
      
      await expect(
        nftReceiver.mint(user1.address, TEST_METADATA_URI)
      ).to.be.revertedWithCustomError(nftReceiver, 'EnforcedPause');

      await nftReceiver.unpause();
      
      // Should work again
      await nftReceiver.mint(user1.address, TEST_METADATA_URI);
    });

    it('should allow owner to mark messages as processed', async () => {
      const messageId = ethers.hexlify(ethers.randomBytes(32));
      
      expect(await nftReceiver.isMessageProcessed(messageId)).to.be.false;
      
      await nftReceiver.adminMarkMessageProcessed(messageId);
      
      expect(await nftReceiver.isMessageProcessed(messageId)).to.be.true;
    });
  });

  describe('View Functions', () => {
    it('should return correct token origin info', async () => {
      await nftReceiver.mint(user1.address, TEST_METADATA_URI);
      
      const tokenOrigin = await nftReceiver.getTokenOrigin(0);
      expect(tokenOrigin.tokenId).to.equal(0);
      expect(tokenOrigin.metadataUri).to.equal(TEST_METADATA_URI);
      expect(tokenOrigin.originChain).to.equal(CHAIN_IDS.BASE_SEPOLIA);
    });

    it('should revert for non-existent token', async () => {
      await expect(
        nftReceiver.getTokenOrigin(999)
      ).to.be.revertedWith('Token does not exist');
    });

    it('should track user nonces correctly', async () => {
      expect(await nftReceiver.getUserNonce(user1.address)).to.equal(0);
      
      // Mint and burn to increment nonce
      await nftReceiver.mint(user1.address, TEST_METADATA_URI);
      await nftReceiver.connect(user1).burnAndTransferToZetaChain(0, user2.address);
      
      expect(await nftReceiver.getUserNonce(user1.address)).to.equal(1);
    });
  });

  describe('Error Handling', () => {
    it('should handle fallback and receive functions', async () => {
      // Send ETH to contract
      const tx = await user1.sendTransaction({
        to: await nftReceiver.getAddress(),
        value: ethers.parseEther('1.0')
      });
      await tx.wait();

      // Verify contract received ETH
      expect(await ethers.provider.getBalance(await nftReceiver.getAddress())).to.equal(
        ethers.parseEther('1.0')
      );

      // Owner should be able to withdraw
      await nftReceiver.withdrawETH();
      expect(await ethers.provider.getBalance(await nftReceiver.getAddress())).to.equal(0);
    });
  });
});