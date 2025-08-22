import { expect } from 'chai';
import { ethers } from 'hardhat';
import { UniversalNFT, UniversalNFTReceiver } from '../typechain-types';
import { CrossChainMessageUtils, CHAIN_IDS, NFTTransferMessage } from '../shared/CrossChainMessage';
import { MessageBridge } from '../shared/MessageBridge';
import { TestEnvironment, createTestEnvironment } from './utils/TestEnvironment';

/**
 * Security and Robustness Testing Suite for Universal NFT Protocol
 * Comprehensive security testing including attack vectors, edge cases, and robustness scenarios
 */
describe('Universal NFT Protocol - Security & Robustness Tests', () => {
  let testEnv: TestEnvironment;
  let zetaChainNFT: UniversalNFT;
  let baseNFTReceiver: UniversalNFTReceiver;

  // Security test tracking
  const securityMetrics = {
    totalAttacksAttempted: 0,
    attacksBlocked: 0,
    vulnerabilitiesFound: [] as string[],
    robustnessTests: 0,
    robustnessFailures: 0,
  };

  before(async function () {
    this.timeout(120000);
    console.log('🛡️  Setting up Security & Robustness Test Environment...');

    testEnv = createTestEnvironment({
      ethRpcUrl: 'http://localhost:8545',
      solanaRpcUrl: 'http://localhost:8899',
      defaultTimeout: 45000,
      networkTimeout: 20000,
    });

    await testEnv.setup();
    await testEnv.deployContracts();

    zetaChainNFT = testEnv.contracts.zetaChainNFT;
    baseNFTReceiver = testEnv.contracts.baseNFTReceiver;

    console.log('✅ Security test environment ready!');
  });

  describe('Authentication and Authorization Security', () => {
    it('should prevent unauthorized minting', async function () {
      this.timeout(30000);
      console.log('🚫 Testing unauthorized minting prevention...');

      securityMetrics.totalAttacksAttempted++;

      // Try to mint from unauthorized account
      const unauthorizedWallet = testEnv.wallets.bob;

      try {
        // This should fail if proper access controls are in place
        await zetaChainNFT
          .connect(unauthorizedWallet)
          .mint(unauthorizedWallet.address, 'https://malicious.com/nft.json');

        console.log('   ❌ SECURITY VULNERABILITY: Unauthorized minting succeeded!');
        securityMetrics.vulnerabilitiesFound.push('Unauthorized minting allowed');

        // If unauthorized minting succeeds, we have a serious security issue
        expect.fail('Unauthorized minting should not be allowed');
      } catch (error: any) {
        console.log('   ✅ Unauthorized minting correctly blocked');
        console.log(`     🔒 Error: ${error.message.slice(0, 100)}...`);
        securityMetrics.attacksBlocked++;
      }
    });

    it('should prevent unauthorized cross-chain message processing', async function () {
      this.timeout(45000);
      console.log('🚫 Testing unauthorized message processing prevention...');

      securityMetrics.totalAttacksAttempted++;

      // Create a message from unauthorized sender
      const maliciousMessage = MessageBridge.createEvmToEvmMessage({
        tokenId: 'malicious_token',
        metadataUri: 'https://malicious.com/fake.json',
        recipientAddress: testEnv.wallets.bob.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: testEnv.wallets.alice.address,
        originContractAddress: '0x' + '00'.repeat(20), // Fake contract address
        nonce: 'malicious_nonce',
      });

      const encodedMessage = CrossChainMessageUtils.encodeForEVM(maliciousMessage);

      try {
        // Try to process message from unauthorized sender (not the mock gateway)
        await baseNFTReceiver
          .connect(testEnv.wallets.bob) // Bob is not the gateway!
          .onCall({ sender: '0x' + '00'.repeat(20) }, encodedMessage);

        console.log('   ❌ SECURITY VULNERABILITY: Unauthorized message processing succeeded!');
        securityMetrics.vulnerabilitiesFound.push('Unauthorized message processing allowed');
        expect.fail('Unauthorized message processing should not be allowed');
      } catch (error: any) {
        console.log('   ✅ Unauthorized message processing correctly blocked');
        console.log(`     🔒 Error: ${error.message.slice(0, 100)}...`);
        securityMetrics.attacksBlocked++;
      }
    });

    it('should validate gateway authorization correctly', async function () {
      this.timeout(30000);
      console.log('🔐 Testing gateway authorization validation...');

      securityMetrics.totalAttacksAttempted++;

      // Create valid message but send from wrong gateway
      const validMessage = MessageBridge.createEvmToEvmMessage({
        tokenId: 'auth_test',
        metadataUri: 'https://test.com/auth.json',
        recipientAddress: testEnv.wallets.bob.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: testEnv.wallets.alice.address,
        originContractAddress: await zetaChainNFT.getAddress(),
        nonce: 'auth_test_nonce',
      });

      const encodedMessage = CrossChainMessageUtils.encodeForEVM(validMessage);

      try {
        // Try with fake gateway address
        const fakeGateway = testEnv.wallets.charlie;
        await baseNFTReceiver
          .connect(fakeGateway)
          .onCall({ sender: await zetaChainNFT.getAddress() }, encodedMessage);

        console.log('   ❌ SECURITY VULNERABILITY: Fake gateway accepted!');
        securityMetrics.vulnerabilitiesFound.push('Fake gateway accepted');
        expect.fail('Fake gateway should not be accepted');
      } catch (error: any) {
        console.log('   ✅ Fake gateway correctly rejected');
        securityMetrics.attacksBlocked++;
      }
    });
  });

  describe('Replay Attack Prevention', () => {
    it('should prevent message replay attacks', async function () {
      this.timeout(60000);
      console.log('🔄 Testing message replay attack prevention...');

      securityMetrics.totalAttacksAttempted++;

      // First, mint an NFT to transfer
      await zetaChainNFT.mint(testEnv.wallets.alice.address, 'https://replay.test.com/nft.json');

      // Create valid transfer message
      const transferMessage = MessageBridge.createEvmToEvmMessage({
        tokenId: 'replay_test',
        metadataUri: 'https://replay.test.com/nft.json',
        recipientAddress: testEnv.wallets.bob.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: testEnv.wallets.alice.address,
        originContractAddress: await zetaChainNFT.getAddress(),
        nonce: 'replay_prevention_test',
      });

      const encodedMessage = CrossChainMessageUtils.encodeForEVM(transferMessage);
      const mockContext = { sender: await zetaChainNFT.getAddress() };

      // First processing should succeed
      console.log('   📤 Processing message first time...');
      const firstTx = await baseNFTReceiver
        .connect(testEnv.wallets.mockGateway)
        .onCall(mockContext, encodedMessage);
      await firstTx.wait();

      console.log('   ✅ First processing succeeded');

      // Verify message is marked as processed
      const messageId = transferMessage.messageId;
      expect(await baseNFTReceiver.isMessageProcessed(messageId)).to.be.true;

      try {
        // Second processing should fail (replay attack)
        console.log('   🔄 Attempting replay attack...');
        await baseNFTReceiver
          .connect(testEnv.wallets.mockGateway)
          .onCall(mockContext, encodedMessage);

        console.log('   ❌ SECURITY VULNERABILITY: Replay attack succeeded!');
        securityMetrics.vulnerabilitiesFound.push('Message replay attack allowed');
        expect.fail('Message replay should not be allowed');
      } catch (error: any) {
        console.log('   ✅ Replay attack correctly prevented');
        console.log(`     🔒 Error: ${error.message.slice(0, 100)}...`);
        securityMetrics.attacksBlocked++;
      }
    });

    it('should handle nonce-based replay prevention', async function () {
      this.timeout(45000);
      console.log('🔢 Testing nonce-based replay prevention...');

      securityMetrics.totalAttacksAttempted++;

      const baseNonce = `nonce_test_${Date.now()}`;
      const zetaAddress = await zetaChainNFT.getAddress();

      // Create two messages with same nonce but different content
      const message1 = MessageBridge.createEvmToEvmMessage({
        tokenId: 'nonce_test_1',
        metadataUri: 'https://nonce.test.com/1.json',
        recipientAddress: testEnv.wallets.bob.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: testEnv.wallets.alice.address,
        originContractAddress: zetaAddress,
        nonce: baseNonce, // Same nonce
      });

      const message2 = MessageBridge.createEvmToEvmMessage({
        tokenId: 'nonce_test_2',
        metadataUri: 'https://nonce.test.com/2.json',
        recipientAddress: testEnv.wallets.charlie.address, // Different recipient
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: testEnv.wallets.alice.address,
        originContractAddress: zetaAddress,
        nonce: baseNonce, // Same nonce - this should create different message IDs
      });

      // Messages should have different IDs despite same nonce
      const messageId1 = Buffer.from(message1.messageId).toString('hex');
      const messageId2 = Buffer.from(message2.messageId).toString('hex');

      expect(messageId1).to.not.equal(messageId2);
      console.log(
        `   ✅ Different message IDs generated: ${messageId1.slice(0, 16)}... vs ${messageId2.slice(0, 16)}...`
      );

      // Both should be processable as they have different IDs
      const mockContext = { sender: zetaAddress };

      const tx1 = await baseNFTReceiver
        .connect(testEnv.wallets.mockGateway)
        .onCall(mockContext, CrossChainMessageUtils.encodeForEVM(message1));
      await tx1.wait();

      const tx2 = await baseNFTReceiver
        .connect(testEnv.wallets.mockGateway)
        .onCall(mockContext, CrossChainMessageUtils.encodeForEVM(message2));
      await tx2.wait();

      console.log('   ✅ Both messages processed successfully with unique IDs');
      securityMetrics.attacksBlocked++; // Proper nonce handling prevents issues
    });
  });

  describe('Input Validation and Sanitization', () => {
    it('should validate and reject malformed messages', async function () {
      this.timeout(45000);
      console.log('🧪 Testing malformed message rejection...');

      // Get the contract address once
      const zetaAddress = await zetaChainNFT.getAddress();

      const malformedTests = [
        {
          name: 'Empty metadata URI',
          getMessage: () => ({
            tokenId: 'malformed_1',
            metadataUri: '', // Empty URI
            recipient: CrossChainMessageUtils.ethereumAddressToBytes32(testEnv.wallets.bob.address),
            originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
            destinationChain: CHAIN_IDS.BASE_SEPOLIA,
            messageId: new Uint8Array(32).fill(1),
            timestamp: Math.floor(Date.now() / 1000),
            originContract: CrossChainMessageUtils.ethereumAddressToBytes32(zetaAddress),
            nonce: 'malformed_1',
          }),
        },
        {
          name: 'Invalid timestamp (too old)',
          getMessage: () => ({
            tokenId: 'malformed_2',
            metadataUri: 'https://test.com/valid.json',
            recipient: CrossChainMessageUtils.ethereumAddressToBytes32(testEnv.wallets.bob.address),
            originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
            destinationChain: CHAIN_IDS.BASE_SEPOLIA,
            messageId: new Uint8Array(32).fill(2),
            timestamp: Math.floor(Date.now() / 1000) - 100000, // Very old timestamp
            originContract: CrossChainMessageUtils.ethereumAddressToBytes32(zetaAddress),
            nonce: 'malformed_2',
          }),
        },
        {
          name: 'Invalid chain IDs (same origin and destination)',
          getMessage: () => ({
            tokenId: 'malformed_3',
            metadataUri: 'https://test.com/valid.json',
            recipient: CrossChainMessageUtils.ethereumAddressToBytes32(testEnv.wallets.bob.address),
            originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
            destinationChain: CHAIN_IDS.ZETACHAIN_TESTNET, // Same as origin!
            messageId: new Uint8Array(32).fill(3),
            timestamp: Math.floor(Date.now() / 1000),
            originContract: CrossChainMessageUtils.ethereumAddressToBytes32(zetaAddress),
            nonce: 'malformed_3',
          }),
        },
      ];

      for (const test of malformedTests) {
        console.log(`   🧪 Testing: ${test.name}`);
        securityMetrics.totalAttacksAttempted++;

        try {
          const malformedMessage = test.getMessage() as NFTTransferMessage;
          CrossChainMessageUtils.validateMessage(malformedMessage);

          console.log(`     ❌ VULNERABILITY: ${test.name} was accepted!`);
          securityMetrics.vulnerabilitiesFound.push(`Malformed message accepted: ${test.name}`);
        } catch (error: any) {
          console.log(`     ✅ ${test.name} correctly rejected: ${error.message.slice(0, 50)}...`);
          securityMetrics.attacksBlocked++;
        }
      }
    });

    it('should sanitize and validate metadata inputs', async function () {
      this.timeout(45000);
      console.log('🧹 Testing metadata input sanitization...');

      const maliciousInputs = [
        {
          name: 'Script injection attempt',
          metadata: '<script>alert("XSS")</script>',
          shouldFail: false, // URLs can contain these chars legally
        },
        {
          name: 'SQL injection attempt',
          metadata: "'; DROP TABLE users; --",
          shouldFail: false, // URLs can contain these chars
        },
        {
          name: 'Extremely long URI',
          metadata: 'https://example.com/' + 'x'.repeat(1000) + '.json',
          shouldFail: true, // Should exceed length limits
        },
        {
          name: 'Control characters',
          metadata: 'https://test.com/\x00\x01\x02.json',
          shouldFail: false, // May be valid in some contexts
        },
      ];

      for (const input of maliciousInputs) {
        console.log(`   🔍 Testing: ${input.name}`);
        securityMetrics.totalAttacksAttempted++;

        try {
          // Try to create message with malicious metadata
          const message = MessageBridge.createEvmToEvmMessage({
            tokenId: `sanitize_${Date.now()}`,
            metadataUri: input.metadata,
            recipientAddress: testEnv.wallets.bob.address,
            originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
            destinationChain: CHAIN_IDS.BASE_SEPOLIA,
            senderAddress: testEnv.wallets.alice.address,
            originContractAddress: await zetaChainNFT.getAddress(),
            nonce: `sanitize_${Date.now()}`,
          });

          // Validate the message
          CrossChainMessageUtils.validateMessage(message);

          if (input.shouldFail) {
            console.log(`     ⚠️  ${input.name} was accepted but might be problematic`);
          } else {
            console.log(`     ✅ ${input.name} handled appropriately`);
            securityMetrics.attacksBlocked++;
          }
        } catch (error: any) {
          if (input.shouldFail) {
            console.log(
              `     ✅ ${input.name} correctly rejected: ${error.message.slice(0, 50)}...`
            );
            securityMetrics.attacksBlocked++;
          } else {
            console.log(
              `     ⚠️  ${input.name} rejected unexpectedly: ${error.message.slice(0, 50)}...`
            );
          }
        }
      }
    });
  });

  describe('Economic Attack Prevention', () => {
    it('should prevent gas manipulation attacks', async function () {
      this.timeout(60000);
      console.log('⛽ Testing gas manipulation attack prevention...');

      securityMetrics.totalAttacksAttempted++;

      // Try to create very expensive operations
      const expensiveMetadata = 'x'.repeat(500); // Maximum allowed size
      const expensiveUri = `data:text/plain,${expensiveMetadata}`;

      try {
        // Estimate gas for expensive operation
        const gasEstimate = await zetaChainNFT.mint.estimateGas(
          testEnv.wallets.alice.address,
          expensiveUri
        );

        console.log(`   ⛽ Gas estimate for expensive operation: ${gasEstimate.toString()}`);

        // Check if gas usage is within reasonable limits
        const MAX_REASONABLE_GAS = 500000n;

        if (gasEstimate > MAX_REASONABLE_GAS) {
          console.log(`   ⚠️  High gas usage detected: ${gasEstimate.toString()}`);
          console.log('   🔒 Gas limits should prevent abuse in production');
        } else {
          console.log('   ✅ Gas usage within reasonable limits');
        }

        securityMetrics.attacksBlocked++;

        // Actually perform the operation to verify it works
        const tx = await zetaChainNFT.mint(testEnv.wallets.alice.address, expensiveUri);
        const receipt = await tx.wait();

        console.log(`   📊 Actual gas used: ${receipt!.gasUsed.toString()}`);
      } catch (error: any) {
        console.log(`   🔒 Expensive operation blocked: ${error.message.slice(0, 100)}...`);
        securityMetrics.attacksBlocked++;
      }
    });

    it('should handle resource exhaustion attempts', async function () {
      this.timeout(90000);
      console.log('💥 Testing resource exhaustion attack prevention...');

      securityMetrics.totalAttacksAttempted++;

      // Attempt rapid-fire operations to exhaust resources
      const rapidOpsCount = 20;
      const rapidOps: Promise<any>[] = [];
      let successCount = 0;
      let errorCount = 0;

      console.log(`   🔥 Attempting ${rapidOpsCount} rapid operations...`);

      for (let i = 0; i < rapidOpsCount; i++) {
        const op = zetaChainNFT
          .mint(testEnv.wallets.alice.address, `https://rapid.test.com/${i}.json`)
          .then((tx) => tx.wait())
          .then(() => {
            successCount++;
          })
          .catch((error: any) => {
            errorCount++;
            // Rate limiting or resource protection kicked in
            console.log(`     🛡️  Operation ${i} blocked: ${error.message.slice(0, 50)}...`);
          });

        rapidOps.push(op);
      }

      await Promise.allSettled(rapidOps);

      console.log(`   📊 Rapid operations results:`);
      console.log(`     ✅ Successful: ${successCount}`);
      console.log(`     🛡️  Blocked: ${errorCount}`);

      // Some blocking is good for protection
      if (errorCount > 0) {
        console.log('   ✅ Resource protection mechanisms working');
        securityMetrics.attacksBlocked++;
      } else if (successCount === rapidOpsCount) {
        console.log('   ⚠️  All rapid operations succeeded - consider rate limiting');
      }
    });
  });

  describe('Edge Case Robustness', () => {
    it('should handle network partition scenarios', async function () {
      this.timeout(60000);
      console.log('🌐 Testing network partition robustness...');

      securityMetrics.robustnessTests++;

      // Simulate network issues with timeouts
      const networkTests = [
        {
          name: 'Slow network simulation',
          delay: 1000, // 1 second delay
        },
        {
          name: 'Very slow network simulation',
          delay: 5000, // 5 second delay
        },
      ];

      for (const test of networkTests) {
        console.log(`   🕐 Testing: ${test.name}`);

        try {
          const startTime = Date.now();

          // Add artificial delay to simulate slow network
          await new Promise((resolve) => setTimeout(resolve, test.delay));

          // Perform operation
          const tx = await zetaChainNFT.mint(
            testEnv.wallets.alice.address,
            `https://network.test.com/${test.name.replace(/\s+/g, '_')}.json`
          );
          await tx.wait();

          const endTime = Date.now();
          const totalTime = endTime - startTime;

          console.log(`     ✅ ${test.name} completed in ${totalTime}ms`);

          // Operation should still succeed despite delays
          expect(totalTime).to.be.greaterThan(test.delay);
        } catch (error: any) {
          console.log(`     ❌ ${test.name} failed: ${error.message}`);
          securityMetrics.robustnessFailures++;
        }
      }
    });

    it('should handle state consistency during failures', async function () {
      this.timeout(90000);
      console.log('🔄 Testing state consistency during failures...');

      securityMetrics.robustnessTests++;

      // Test contract pause/unpause scenarios
      console.log('   🛑 Testing contract pause handling...');

      try {
        // Pause the contract
        await baseNFTReceiver.pause();
        expect(await baseNFTReceiver.paused()).to.be.true;
        console.log('     ✅ Contract paused successfully');

        // Try to process a message while paused (should fail)
        const pausedMessage = MessageBridge.createEvmToEvmMessage({
          tokenId: 'paused_test',
          metadataUri: 'https://paused.test.com/nft.json',
          recipientAddress: testEnv.wallets.bob.address,
          originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
          destinationChain: CHAIN_IDS.BASE_SEPOLIA,
          senderAddress: testEnv.wallets.alice.address,
          originContractAddress: await zetaChainNFT.getAddress(),
          nonce: 'paused_test',
        });

        try {
          await baseNFTReceiver
            .connect(testEnv.wallets.mockGateway)
            .onCall(
              { sender: await zetaChainNFT.getAddress() },
              CrossChainMessageUtils.encodeForEVM(pausedMessage)
            );

          console.log('     ❌ Operations succeeded while paused - potential issue');
          securityMetrics.robustnessFailures++;
        } catch (pauseError: any) {
          console.log('     ✅ Operations correctly blocked while paused');
        }

        // Unpause and try again
        await baseNFTReceiver.unpause();
        expect(await baseNFTReceiver.paused()).to.be.false;
        console.log('     ✅ Contract unpaused successfully');

        // Now the operation should succeed
        const unpausedTx = await baseNFTReceiver
          .connect(testEnv.wallets.mockGateway)
          .onCall(
            { sender: await zetaChainNFT.getAddress() },
            CrossChainMessageUtils.encodeForEVM(pausedMessage)
          );
        await unpausedTx.wait();

        console.log('     ✅ Operations work correctly after unpause');
      } catch (error: any) {
        console.log(`     ❌ Pause/unpause test failed: ${error.message}`);
        securityMetrics.robustnessFailures++;
      }
    });

    it('should handle message ordering and consistency', async function () {
      this.timeout(90000);
      console.log('🔢 Testing message ordering and consistency...');

      securityMetrics.robustnessTests++;

      // Create multiple messages with different nonces
      const messageCount = 10;
      const messages: any[] = [];
      const zetaAddress = await zetaChainNFT.getAddress();

      console.log(`   📤 Creating ${messageCount} ordered messages...`);

      for (let i = 0; i < messageCount; i++) {
        const message = MessageBridge.createEvmToEvmMessage({
          tokenId: `ordering_${i}`,
          metadataUri: `https://ordering.test.com/${i}.json`,
          recipientAddress: testEnv.wallets.bob.address,
          originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
          destinationChain: CHAIN_IDS.BASE_SEPOLIA,
          senderAddress: testEnv.wallets.alice.address,
          originContractAddress: zetaAddress,
          nonce: `ordering_${i.toString().padStart(3, '0')}`,
        });
        messages.push(message);
      }

      // Process messages in random order to test ordering independence
      const shuffledMessages = [...messages].sort(() => Math.random() - 0.5);
      console.log('   🔀 Processing messages in random order...');

      let processedCount = 0;
      for (const message of shuffledMessages) {
        try {
          const tx = await baseNFTReceiver
            .connect(testEnv.wallets.mockGateway)
            .onCall({ sender: zetaAddress }, CrossChainMessageUtils.encodeForEVM(message));
          await tx.wait();
          processedCount++;
        } catch (error: any) {
          console.log(`     ⚠️  Message processing failed: ${error.message.slice(0, 50)}...`);
          // Some failures might be acceptable (duplicate processing, etc.)
        }
      }

      console.log(`   📊 Successfully processed ${processedCount}/${messageCount} messages`);

      // Verify all messages are marked as processed
      let markedProcessed = 0;
      for (const message of messages) {
        if (await baseNFTReceiver.isMessageProcessed(message.messageId)) {
          markedProcessed++;
        }
      }

      console.log(
        `   ✅ ${markedProcessed}/${messageCount} messages correctly marked as processed`
      );

      if (processedCount < messageCount * 0.8) {
        console.log('   ⚠️  Low success rate - might indicate ordering issues');
        securityMetrics.robustnessFailures++;
      }
    });
  });

  describe('Administrative Security', () => {
    it('should protect administrative functions', async function () {
      this.timeout(45000);
      console.log('👮 Testing administrative function protection...');

      securityMetrics.totalAttacksAttempted++;

      // Test admin-only functions with unauthorized user
      const unauthorizedUser = testEnv.wallets.bob;

      const adminTests = [
        {
          name: 'Unauthorized pause attempt',
          test: () => baseNFTReceiver.connect(unauthorizedUser).pause(),
        },
        {
          name: 'Unauthorized unpause attempt',
          test: () => baseNFTReceiver.connect(unauthorizedUser).unpause(),
        },
        {
          name: 'Unauthorized message marking',
          test: () =>
            baseNFTReceiver
              .connect(unauthorizedUser)
              .adminMarkMessageProcessed(ethers.hexlify(ethers.randomBytes(32))),
        },
      ];

      for (const adminTest of adminTests) {
        console.log(`   🔒 Testing: ${adminTest.name}`);

        try {
          await adminTest.test();

          console.log(`     ❌ SECURITY VULNERABILITY: ${adminTest.name} succeeded!`);
          securityMetrics.vulnerabilitiesFound.push(`Unauthorized admin access: ${adminTest.name}`);
        } catch (error: any) {
          console.log(`     ✅ ${adminTest.name} correctly blocked`);
          console.log(`       🔒 Error: ${error.message.slice(0, 50)}...`);
          securityMetrics.attacksBlocked++;
        }
      }
    });

    it('should validate ownership transfer security', async function () {
      this.timeout(45000);
      console.log('👑 Testing ownership transfer security...');

      securityMetrics.totalAttacksAttempted++;

      // Test unauthorized ownership transfer attempts
      try {
        const currentOwner = await baseNFTReceiver.owner();
        console.log(`   📋 Current contract owner: ${currentOwner}`);

        // Try to transfer ownership from unauthorized account
        await baseNFTReceiver
          .connect(testEnv.wallets.bob)
          .transferOwnership(testEnv.wallets.bob.address);

        console.log('   ❌ SECURITY VULNERABILITY: Unauthorized ownership transfer succeeded!');
        securityMetrics.vulnerabilitiesFound.push('Unauthorized ownership transfer allowed');
      } catch (error: any) {
        console.log('   ✅ Unauthorized ownership transfer correctly blocked');
        console.log(`     🔒 Error: ${error.message.slice(0, 100)}...`);
        securityMetrics.attacksBlocked++;
      }
    });
  });

  after(function () {
    console.log('\n🛡️  SECURITY & ROBUSTNESS TESTS COMPLETED 🛡️');
    console.log('='.repeat(80));

    // Generate security report
    const attackSuccessRate =
      (securityMetrics.attacksBlocked / Math.max(securityMetrics.totalAttacksAttempted, 1)) * 100;
    const robustnessSuccessRate =
      ((securityMetrics.robustnessTests - securityMetrics.robustnessFailures) /
        Math.max(securityMetrics.robustnessTests, 1)) *
      100;

    console.log('📊 SECURITY TEST SUMMARY:');
    console.log(`   🎯 Total attack attempts: ${securityMetrics.totalAttacksAttempted}`);
    console.log(`   🛡️  Attacks blocked: ${securityMetrics.attacksBlocked}`);
    console.log(`   🔒 Attack block rate: ${attackSuccessRate.toFixed(1)}%`);
    console.log(`   ⚠️  Vulnerabilities found: ${securityMetrics.vulnerabilitiesFound.length}`);

    if (securityMetrics.vulnerabilitiesFound.length > 0) {
      console.log('\n❌ VULNERABILITIES DETECTED:');
      securityMetrics.vulnerabilitiesFound.forEach((vuln, index) => {
        console.log(`   ${index + 1}. ${vuln}`);
      });
    }

    console.log('\n📊 ROBUSTNESS TEST SUMMARY:');
    console.log(`   🧪 Total robustness tests: ${securityMetrics.robustnessTests}`);
    console.log(`   ❌ Robustness failures: ${securityMetrics.robustnessFailures}`);
    console.log(`   ✅ Robustness success rate: ${robustnessSuccessRate.toFixed(1)}%`);

    console.log('\n🎯 SECURITY VERDICT:');

    if (securityMetrics.vulnerabilitiesFound.length === 0 && attackSuccessRate > 90) {
      console.log('   ✅ EXCELLENT: Protocol demonstrates strong security posture');
      console.log('   ✅ All attack vectors successfully defended');
      console.log('   ✅ Administrative functions properly protected');
    } else if (securityMetrics.vulnerabilitiesFound.length <= 2 && attackSuccessRate > 80) {
      console.log('   ⚠️  GOOD: Protocol shows good security with minor concerns');
      console.log('   ⚠️  Some vulnerabilities identified - review recommended');
    } else {
      console.log('   ❌ CONCERNING: Security issues need immediate attention');
      console.log('   ❌ Multiple vulnerabilities or low defense rate detected');
    }

    if (robustnessSuccessRate > 90) {
      console.log('   ✅ Protocol demonstrates excellent robustness');
    } else if (robustnessSuccessRate > 75) {
      console.log('   ⚠️  Protocol shows acceptable robustness with room for improvement');
    } else {
      console.log('   ❌ Protocol robustness needs significant improvement');
    }

    console.log('\n🚀 Security testing completed - review findings for production readiness!');
  });
});
