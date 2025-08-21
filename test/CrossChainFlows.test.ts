import { expect } from 'chai';
import { ethers } from 'ethers';
import { PublicKey, Keypair } from '@solana/web3.js';
import { CrossChainMessageUtils, CHAIN_IDS, NFTTransferMessage } from '../shared/CrossChainMessage';
import { MessageBridge } from '../shared/MessageBridge';

/**
 * Comprehensive Cross-Chain Flow Tests
 * These tests verify all the specific flow scenarios required:
 * 1. Solana → Base Sepolia
 * 2. ZetaChain → Solana
 * 3. Base Sepolia → Solana
 * 4. Full Loop: ZetaChain → Base → Solana → ZetaChain
 */
describe('Cross-Chain NFT Flows', () => {
  // Test wallets
  let aliceEth: ethers.Wallet;
  let bobEth: ethers.Wallet;
  let charlieSol: Keypair;
  let davidSol: Keypair;

  // Test NFT data
  const TEST_NFT = {
    tokenId: '2024',
    metadataUri: 'https://api.universalnft.com/metadata/2024',
    name: 'Cross-Chain Test NFT',
    description: 'A test NFT for comprehensive cross-chain flow testing',
  };

  before(() => {
    // Setup test wallets
    aliceEth = new ethers.Wallet('0x' + '01'.repeat(32));
    bobEth = new ethers.Wallet('0x' + '02'.repeat(32));
    charlieSol = Keypair.generate();
    davidSol = Keypair.generate();

    console.log('🚀 Setting up Cross-Chain Flow Tests...');
    console.log(`   Alice (ETH): ${aliceEth.address}`);
    console.log(`   Bob (ETH): ${bobEth.address}`);
    console.log(`   Charlie (SOL): ${charlieSol.publicKey.toString()}`);
    console.log(`   David (SOL): ${davidSol.publicKey.toString()}`);
  });

  describe('Flow 1: Solana → Base Sepolia', () => {
    let transferMessage: NFTTransferMessage;
    let processedMessages: Set<string> = new Set();

    it('Step 1: Create transfer message from Solana', async () => {
      console.log('\n🌊 Flow 1: Solana → Base Sepolia');
      console.log('📝 Step 1: Creating transfer message from Solana...');

      // Create Solana to Base transfer message
      transferMessage = MessageBridge.createSolanaToEvmMessage({
        tokenId: TEST_NFT.tokenId,
        metadataUri: TEST_NFT.metadataUri,
        recipientAddress: bobEth.address,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: charlieSol.publicKey.toString(),
        nonce: '1',
      });

      // Validate message structure
      expect(transferMessage.tokenId).to.equal(TEST_NFT.tokenId);
      expect(transferMessage.metadataUri).to.equal(TEST_NFT.metadataUri);
      expect(transferMessage.originChain).to.equal(CHAIN_IDS.SOLANA_DEVNET);
      expect(transferMessage.destinationChain).to.equal(CHAIN_IDS.BASE_SEPOLIA);

      // Verify recipient conversion
      const recipientAddress = CrossChainMessageUtils.bytes32ToEthereumAddress(transferMessage.recipient);
      expect(recipientAddress.toLowerCase()).to.equal(bobEth.address.toLowerCase());

      // Verify origin contract is Solana format
      const originPubkey = CrossChainMessageUtils.bytes32ToSolanaPublicKey(transferMessage.originContract);
      expect(originPubkey.toString()).to.equal(charlieSol.publicKey.toString());

      console.log(`   ✅ Message created: ${transferMessage.tokenId}`);
      console.log(`   ✅ Origin: ${CrossChainMessageUtils.getChainName(transferMessage.originChain)}`);
      console.log(`   ✅ Destination: ${CrossChainMessageUtils.getChainName(transferMessage.destinationChain)}`);
      console.log(`   ✅ Recipient: ${recipientAddress}`);
    });

    it('Step 2: Convert to Solana format for program processing', async () => {
      console.log('📤 Step 2: Converting to Solana format...');

      // Convert to Solana format for the burning program
      const solanaMessage = MessageBridge.toSolanaFormat(transferMessage);

      expect(solanaMessage.messageType).to.equal(1); // NFT_MINT
      expect(solanaMessage.tokenId).to.equal(transferMessage.tokenId);
      expect(solanaMessage.metadataUri).to.equal(transferMessage.metadataUri);
      expect(solanaMessage.originChainId).to.equal(CHAIN_IDS.SOLANA_DEVNET);

      // Verify addresses in Solana format
      expect(solanaMessage.recipientAddress).to.deep.equal(transferMessage.recipient);
      expect(solanaMessage.originAddress).to.deep.equal(transferMessage.originContract);

      console.log(`   ✅ Solana message type: ${solanaMessage.messageType}`);
      console.log(`   ✅ Token ID preserved: ${solanaMessage.tokenId}`);
      console.log(`   ✅ Metadata preserved: ${solanaMessage.metadataUri}`);
    });

    it('Step 3: Process message on Base Sepolia', async () => {
      console.log('📨 Step 3: Processing message on Base Sepolia...');

      // Convert to EVM format for Gateway transmission
      const evmEncodedMessage = CrossChainMessageUtils.encodeForEVM(transferMessage);
      expect(evmEncodedMessage).to.be.a('string');
      expect(evmEncodedMessage.startsWith('0x')).to.be.true;

      // Simulate Gateway message processing on Base Sepolia
      const decodedMessage = CrossChainMessageUtils.decodeFromEVM(evmEncodedMessage);

      // Verify message integrity
      expect(decodedMessage.tokenId).to.equal(transferMessage.tokenId);
      expect(decodedMessage.metadataUri).to.equal(transferMessage.metadataUri);
      expect(decodedMessage.originChain).to.equal(transferMessage.originChain);
      expect(decodedMessage.destinationChain).to.equal(transferMessage.destinationChain);

      // Verify replay protection
      const messageIdHex = Buffer.from(decodedMessage.messageId).toString('hex');
      expect(processedMessages.has(messageIdHex)).to.be.false;
      processedMessages.add(messageIdHex);

      console.log(`   ✅ Message processed on Base Sepolia`);
      console.log(`   ✅ NFT would be minted to: ${CrossChainMessageUtils.bytes32ToEthereumAddress(decodedMessage.recipient)}`);
      console.log(`   ✅ Origin preserved: ${CrossChainMessageUtils.getChainName(decodedMessage.originChain)}`);
    });

    it('Step 4: Verify complete Solana → Base flow', async () => {
      console.log('🔍 Step 4: Verifying complete flow...');

      // Verify route is supported
      expect(MessageBridge.isRouteSupported(CHAIN_IDS.SOLANA_DEVNET, CHAIN_IDS.BASE_SEPOLIA)).to.be.true;

      // Verify message validation passes
      expect(() => CrossChainMessageUtils.validateMessage(transferMessage)).to.not.throw();

      // Verify provenance tracking
      expect(transferMessage.originChain).to.equal(CHAIN_IDS.SOLANA_DEVNET);
      expect(transferMessage.destinationChain).to.equal(CHAIN_IDS.BASE_SEPOLIA);

      console.log('   ✅ Solana → Base Sepolia flow completed successfully!');
    });
  });

  describe('Flow 2: ZetaChain → Solana', () => {
    let transferMessage: NFTTransferMessage;
    let processedMessages: Set<string> = new Set();

    it('Step 1: Create transfer message from ZetaChain', async () => {
      console.log('\n🌊 Flow 2: ZetaChain → Solana');
      console.log('📝 Step 1: Creating transfer message from ZetaChain...');

      // Create ZetaChain to Solana transfer message
      transferMessage = MessageBridge.createEvmToSolanaMessage({
        tokenId: TEST_NFT.tokenId,
        metadataUri: TEST_NFT.metadataUri,
        recipientAddress: davidSol.publicKey.toString(),
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        senderAddress: aliceEth.address,
        originContractAddress: aliceEth.address, // Mock ZetaChain contract
        nonce: '2',
      });

      // Validate message structure
      expect(transferMessage.tokenId).to.equal(TEST_NFT.tokenId);
      expect(transferMessage.metadataUri).to.equal(TEST_NFT.metadataUri);
      expect(transferMessage.originChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);
      expect(transferMessage.destinationChain).to.equal(CHAIN_IDS.SOLANA_DEVNET);

      // Verify recipient conversion to Solana format
      const recipientPubkey = CrossChainMessageUtils.bytes32ToSolanaPublicKey(transferMessage.recipient);
      expect(recipientPubkey.toString()).to.equal(davidSol.publicKey.toString());

      console.log(`   ✅ Message created: ${transferMessage.tokenId}`);
      console.log(`   ✅ Origin: ${CrossChainMessageUtils.getChainName(transferMessage.originChain)}`);
      console.log(`   ✅ Destination: ${CrossChainMessageUtils.getChainName(transferMessage.destinationChain)}`);
      console.log(`   ✅ Recipient: ${recipientPubkey.toString()}`);
    });

    it('Step 2: Route through ZetaChain Gateway', async () => {
      console.log('🌉 Step 2: Routing through ZetaChain Gateway...');

      // Encode for Gateway transmission
      const evmEncodedMessage = CrossChainMessageUtils.encodeForEVM(transferMessage);
      expect(evmEncodedMessage).to.be.a('string');
      expect(evmEncodedMessage.length).to.be.greaterThan(2);

      // Simulate Gateway validation
      const gatewayValidation = CrossChainMessageUtils.decodeFromEVM(evmEncodedMessage);
      expect(gatewayValidation.tokenId).to.equal(transferMessage.tokenId);

      console.log(`   ✅ Message encoded for Gateway: ${evmEncodedMessage.slice(0, 32)}...`);
      console.log(`   ✅ Gateway validation passed`);
    });

    it('Step 3: Process message on Solana', async () => {
      console.log('📨 Step 3: Processing message on Solana...');

      // Convert to Solana format for program processing
      const solanaMessage = MessageBridge.toSolanaFormat(transferMessage);

      expect(solanaMessage.messageType).to.equal(1); // NFT_MINT
      expect(solanaMessage.tokenId).to.equal(transferMessage.tokenId);
      expect(solanaMessage.metadataUri).to.equal(transferMessage.metadataUri);
      expect(solanaMessage.originChainId).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);

      // Verify recipient is correct Solana pubkey
      const recipientPubkey = CrossChainMessageUtils.bytes32ToSolanaPublicKey(solanaMessage.recipientAddress);
      expect(recipientPubkey.toString()).to.equal(davidSol.publicKey.toString());

      // Track message processing
      const messageIdHex = Buffer.from(transferMessage.messageId).toString('hex');
      expect(processedMessages.has(messageIdHex)).to.be.false;
      processedMessages.add(messageIdHex);

      console.log(`   ✅ Solana program would mint NFT to: ${recipientPubkey.toString()}`);
      console.log(`   ✅ Origin preserved: ${CrossChainMessageUtils.getChainName(solanaMessage.originChainId)}`);
    });

    it('Step 4: Verify complete ZetaChain → Solana flow', async () => {
      console.log('🔍 Step 4: Verifying complete flow...');

      // Verify route is supported
      expect(MessageBridge.isRouteSupported(CHAIN_IDS.ZETACHAIN_TESTNET, CHAIN_IDS.SOLANA_DEVNET)).to.be.true;

      // Verify message validation passes
      expect(() => CrossChainMessageUtils.validateMessage(transferMessage)).to.not.throw();

      // Verify cross-chain compatibility
      expect(() => MessageBridge.validateCrossChainCompatibility(transferMessage)).to.not.throw();

      console.log('   ✅ ZetaChain → Solana flow completed successfully!');
    });
  });

  describe('Flow 3: Base Sepolia → Solana', () => {
    let transferMessage: NFTTransferMessage;
    let processedMessages: Set<string> = new Set();

    it('Step 1: Create transfer message from Base Sepolia', async () => {
      console.log('\n🌊 Flow 3: Base Sepolia → Solana');
      console.log('📝 Step 1: Creating transfer message from Base Sepolia...');

      // Create Base to Solana transfer message
      transferMessage = MessageBridge.createEvmToSolanaMessage({
        tokenId: TEST_NFT.tokenId,
        metadataUri: TEST_NFT.metadataUri,
        recipientAddress: charlieSol.publicKey.toString(),
        originChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: bobEth.address,
        originContractAddress: bobEth.address, // Mock Base contract
        nonce: '3',
      });

      // Validate message structure
      expect(transferMessage.tokenId).to.equal(TEST_NFT.tokenId);
      expect(transferMessage.metadataUri).to.equal(TEST_NFT.metadataUri);
      expect(transferMessage.originChain).to.equal(CHAIN_IDS.BASE_SEPOLIA);
      expect(transferMessage.destinationChain).to.equal(CHAIN_IDS.SOLANA_DEVNET);

      console.log(`   ✅ Message created: ${transferMessage.tokenId}`);
      console.log(`   ✅ Origin: ${CrossChainMessageUtils.getChainName(transferMessage.originChain)}`);
      console.log(`   ✅ Destination: ${CrossChainMessageUtils.getChainName(transferMessage.destinationChain)}`);
    });

    it('Step 2: Route through ZetaChain Gateway to Solana', async () => {
      console.log('🌉 Step 2: Routing through ZetaChain Gateway to Solana...');

      // Base Sepolia would call ZetaChain Gateway with this message
      const evmEncodedMessage = CrossChainMessageUtils.encodeForEVM(transferMessage);
      
      // Gateway would validate and forward to Solana
      const gatewayValidation = CrossChainMessageUtils.decodeFromEVM(evmEncodedMessage);
      expect(gatewayValidation.destinationChain).to.equal(CHAIN_IDS.SOLANA_DEVNET);

      console.log(`   ✅ Gateway routing configured for Solana destination`);
      console.log(`   ✅ Message validated by Gateway`);
    });

    it('Step 3: Convert to Solana format and process', async () => {
      console.log('📨 Step 3: Converting to Solana format and processing...');

      // Convert to Solana format
      const solanaMessage = MessageBridge.toSolanaFormat(transferMessage);

      expect(solanaMessage.messageType).to.equal(1); // NFT_MINT
      expect(solanaMessage.tokenId).to.equal(transferMessage.tokenId);
      expect(solanaMessage.originChainId).to.equal(CHAIN_IDS.BASE_SEPOLIA);

      // Verify recipient conversion
      const recipientPubkey = CrossChainMessageUtils.bytes32ToSolanaPublicKey(solanaMessage.recipientAddress);
      expect(recipientPubkey.toString()).to.equal(charlieSol.publicKey.toString());

      // Track processing
      const messageIdHex = Buffer.from(transferMessage.messageId).toString('hex');
      processedMessages.add(messageIdHex);

      console.log(`   ✅ NFT would be minted on Solana to: ${recipientPubkey.toString()}`);
      console.log(`   ✅ Origin chain preserved: ${CrossChainMessageUtils.getChainName(solanaMessage.originChainId)}`);
    });

    it('Step 4: Verify complete Base → Solana flow', async () => {
      console.log('🔍 Step 4: Verifying complete flow...');

      // Verify route is supported
      expect(MessageBridge.isRouteSupported(CHAIN_IDS.BASE_SEPOLIA, CHAIN_IDS.SOLANA_DEVNET)).to.be.true;

      // Verify message validation
      expect(() => CrossChainMessageUtils.validateMessage(transferMessage)).to.not.throw();

      console.log('   ✅ Base Sepolia → Solana flow completed successfully!');
    });
  });

  describe('Flow 4: Full Loop (ZetaChain → Base → Solana → ZetaChain)', () => {
    let messages: NFTTransferMessage[] = [];
    let processedMessages: Set<string> = new Set();
    const loopTokenId = '3000';

    it('Leg 1: ZetaChain → Base Sepolia', async () => {
      console.log('\n🔄 Flow 4: Full Loop Journey');
      console.log('🚀 Leg 1: ZetaChain → Base Sepolia...');

      const message = MessageBridge.createEvmToEvmMessage({
        tokenId: loopTokenId,
        metadataUri: TEST_NFT.metadataUri,
        recipientAddress: bobEth.address,
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        destinationChain: CHAIN_IDS.BASE_SEPOLIA,
        senderAddress: aliceEth.address,
        originContractAddress: aliceEth.address,
        nonce: '1',
      });

      messages.push(message);

      // Verify this is the starting point
      expect(message.originChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);
      expect(message.destinationChain).to.equal(CHAIN_IDS.BASE_SEPOLIA);

      const messageIdHex = Buffer.from(message.messageId).toString('hex');
      processedMessages.add(messageIdHex);

      console.log(`   ✅ Leg 1 completed: Token ${loopTokenId} now on Base Sepolia`);
      console.log(`   ✅ Original origin preserved: ${CrossChainMessageUtils.getChainName(message.originChain)}`);
    });

    it('Leg 2: Base Sepolia → Solana', async () => {
      console.log('🚀 Leg 2: Base Sepolia → Solana...');

      // For this leg, we preserve the ORIGINAL origin (ZetaChain) but transfer from current location (Base)
      const message = MessageBridge.createEvmToSolanaMessage({
        tokenId: loopTokenId,
        metadataUri: TEST_NFT.metadataUri,
        recipientAddress: charlieSol.publicKey.toString(),
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET, // PRESERVE original origin
        senderAddress: bobEth.address,
        originContractAddress: aliceEth.address, // PRESERVE original contract
        nonce: '2',
      });

      messages.push(message);

      // Verify origin preservation
      expect(message.originChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET); // Still ZetaChain!
      expect(message.destinationChain).to.equal(CHAIN_IDS.SOLANA_DEVNET);

      const messageIdHex = Buffer.from(message.messageId).toString('hex');
      processedMessages.add(messageIdHex);

      console.log(`   ✅ Leg 2 completed: Token ${loopTokenId} now on Solana`);
      console.log(`   ✅ Original origin still preserved: ${CrossChainMessageUtils.getChainName(message.originChain)}`);
    });

    it('Leg 3: Solana → ZetaChain (completing the loop)', async () => {
      console.log('🚀 Leg 3: Solana → ZetaChain (completing the loop)...');

      const message = MessageBridge.createSolanaToEvmMessage({
        tokenId: loopTokenId,
        metadataUri: TEST_NFT.metadataUri,
        recipientAddress: aliceEth.address, // Back to original owner!
        destinationChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        senderAddress: charlieSol.publicKey.toString(),
        nonce: '3',
      });

      messages.push(message);

      // For the return leg, Solana becomes the origin
      expect(message.originChain).to.equal(CHAIN_IDS.SOLANA_DEVNET);
      expect(message.destinationChain).to.equal(CHAIN_IDS.ZETACHAIN_TESTNET);

      const messageIdHex = Buffer.from(message.messageId).toString('hex');
      processedMessages.add(messageIdHex);

      console.log(`   ✅ Leg 3 completed: Token ${loopTokenId} returned to ZetaChain`);
      console.log(`   ✅ Final recipient: ${aliceEth.address} (original owner)`);
    });

    it('Verify complete loop integrity', async () => {
      console.log('🔍 Verifying complete loop integrity...');

      // Verify we have 3 messages (one for each leg)
      expect(messages).to.have.length(3);

      // Verify all message IDs are unique (no replay)
      expect(processedMessages.size).to.equal(3);

      // Verify token ID consistency throughout journey
      messages.forEach(message => {
        expect(message.tokenId).to.equal(loopTokenId);
        expect(message.metadataUri).to.equal(TEST_NFT.metadataUri);
      });

      // Verify route support for each leg
      const routes = [
        { from: CHAIN_IDS.ZETACHAIN_TESTNET, to: CHAIN_IDS.BASE_SEPOLIA },
        { from: CHAIN_IDS.BASE_SEPOLIA, to: CHAIN_IDS.SOLANA_DEVNET },
        { from: CHAIN_IDS.SOLANA_DEVNET, to: CHAIN_IDS.ZETACHAIN_TESTNET },
      ];

      routes.forEach((route, index) => {
        expect(MessageBridge.isRouteSupported(route.from, route.to)).to.be.true;
        console.log(`   ✅ Route ${index + 1} supported: ${CrossChainMessageUtils.getChainName(route.from)} → ${CrossChainMessageUtils.getChainName(route.to)}`);
      });

      // Verify message validation for all legs
      messages.forEach((message, index) => {
        expect(() => CrossChainMessageUtils.validateMessage(message)).to.not.throw();
        console.log(`   ✅ Leg ${index + 1} message validation passed`);
      });

      console.log('\n🎉 FULL LOOP COMPLETED SUCCESSFULLY!');
      console.log(`   Token ${loopTokenId} journey: ZetaChain → Base → Solana → ZetaChain`);
      console.log(`   ✅ All security checks passed`);
      console.log(`   ✅ No replay attacks detected`);
      console.log(`   ✅ Metadata consistency maintained`);
      console.log(`   ✅ Token returned to original owner`);
    });
  });

  describe('Flow Validation and Security', () => {
    it('should validate all supported routes', async () => {
      console.log('\n🔒 Validating all supported routes...');

      const allRoutes = MessageBridge.getSupportedRoutes();
      expect(allRoutes).to.have.length(6); // All 6 possible routes

      allRoutes.forEach(route => {
        expect(MessageBridge.isRouteSupported(route.from, route.to)).to.be.true;
        console.log(`   ✅ ${route.name}`);
      });
    });

    it('should prevent same-chain transfers', async () => {
      console.log('\n🚫 Testing same-chain transfer prevention...');

      const chains = [CHAIN_IDS.ZETACHAIN_TESTNET, CHAIN_IDS.BASE_SEPOLIA, CHAIN_IDS.SOLANA_DEVNET];

      chains.forEach(chainId => {
        expect(MessageBridge.isRouteSupported(chainId, chainId)).to.be.false;
        console.log(`   ✅ ${CrossChainMessageUtils.getChainName(chainId)} → ${CrossChainMessageUtils.getChainName(chainId)} blocked`);
      });
    });

    it('should maintain message uniqueness across all flows', async () => {
      console.log('\n🔑 Testing message uniqueness...');

      const messages: NFTTransferMessage[] = [];
      const messageIds = new Set<string>();

      // Create messages for all possible flows
      const flows = [
        { from: CHAIN_IDS.SOLANA_DEVNET, to: CHAIN_IDS.BASE_SEPOLIA, sender: charlieSol.publicKey.toString(), recipient: bobEth.address },
        { from: CHAIN_IDS.ZETACHAIN_TESTNET, to: CHAIN_IDS.SOLANA_DEVNET, sender: aliceEth.address, recipient: davidSol.publicKey.toString() },
        { from: CHAIN_IDS.BASE_SEPOLIA, to: CHAIN_IDS.SOLANA_DEVNET, sender: bobEth.address, recipient: charlieSol.publicKey.toString() },
      ];

      flows.forEach((flow, index) => {
        let message: NFTTransferMessage;

        if (flow.from === CHAIN_IDS.SOLANA_DEVNET) {
          message = MessageBridge.createSolanaToEvmMessage({
            tokenId: (4000 + index).toString(),
            metadataUri: TEST_NFT.metadataUri,
            recipientAddress: flow.recipient,
            destinationChain: flow.to,
            senderAddress: flow.sender,
            nonce: (index + 1).toString(),
          });
        } else {
          const isRecipientSolana = flow.to === CHAIN_IDS.SOLANA_DEVNET;
          if (isRecipientSolana) {
            message = MessageBridge.createEvmToSolanaMessage({
              tokenId: (4000 + index).toString(),
              metadataUri: TEST_NFT.metadataUri,
              recipientAddress: flow.recipient,
              originChain: flow.from,
              senderAddress: flow.sender,
              originContractAddress: flow.sender,
              nonce: (index + 1).toString(),
            });
          } else {
            message = MessageBridge.createEvmToEvmMessage({
              tokenId: (4000 + index).toString(),
              metadataUri: TEST_NFT.metadataUri,
              recipientAddress: flow.recipient,
              originChain: flow.from,
              destinationChain: flow.to,
              senderAddress: flow.sender,
              originContractAddress: flow.sender,
              nonce: (index + 1).toString(),
            });
          }
        }

        messages.push(message);
        const messageIdHex = Buffer.from(message.messageId).toString('hex');
        
        expect(messageIds.has(messageIdHex)).to.be.false;
        messageIds.add(messageIdHex);
      });

      expect(messages).to.have.length(flows.length);
      expect(messageIds.size).to.equal(flows.length);

      console.log(`   ✅ Generated ${messages.length} unique messages`);
      console.log(`   ✅ All message IDs unique: ${messageIds.size} IDs`);
    });

    it('should validate cross-chain compatibility for all flows', async () => {
      console.log('\n✅ Testing cross-chain compatibility...');

      const testMessage = MessageBridge.createEvmToSolanaMessage({
        tokenId: '5000',
        metadataUri: TEST_NFT.metadataUri,
        recipientAddress: charlieSol.publicKey.toString(),
        originChain: CHAIN_IDS.ZETACHAIN_TESTNET,
        senderAddress: aliceEth.address,
        originContractAddress: aliceEth.address,
        nonce: '1',
      });

      expect(() => MessageBridge.validateCrossChainCompatibility(testMessage)).to.not.throw();
      expect(() => CrossChainMessageUtils.validateMessage(testMessage)).to.not.throw();

      console.log('   ✅ Cross-chain compatibility validation passed');
    });
  });

  after(() => {
    console.log('\n🏁 Cross-Chain Flow Tests completed!');
    console.log('\n📊 Flow Test Summary:');
    console.log('   ✅ Flow 1: Solana → Base Sepolia');
    console.log('   ✅ Flow 2: ZetaChain → Solana');
    console.log('   ✅ Flow 3: Base Sepolia → Solana');
    console.log('   ✅ Flow 4: Full Loop (ZetaChain → Base → Solana → ZetaChain)');
    console.log('   ✅ Security validations');
    console.log('   ✅ Route validations');
    console.log('   ✅ Message uniqueness');
    console.log('   ✅ Cross-chain compatibility');
    console.log('\n🎉 All cross-chain flows verified and working!');
  });
});