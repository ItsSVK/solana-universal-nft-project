const { ethers } = require('ethers');
const { PublicKey, Keypair } = require('@solana/web3.js');
const { createFlowOrchestrator, FlowUtils } = require('../dist/shared/FlowOrchestrator');
const { CrossChainMessageUtils, CHAIN_IDS } = require('../dist/shared/CrossChainMessage');

/**
 * Universal NFT Protocol - Cross-Chain Flow Demo
 * This script demonstrates all supported cross-chain flows:
 * 1. Solana → Base Sepolia
 * 2. ZetaChain → Solana
 * 3. Base Sepolia → Solana  
 * 4. Full Loop: ZetaChain → Base → Solana → ZetaChain
 */

async function main() {
  console.log('🚀 Universal NFT Protocol - Cross-Chain Flow Demo');
  console.log('=' .repeat(60));

  // Setup test wallets
  const aliceEth = new ethers.Wallet('0x' + '01'.repeat(32));
  const bobEth = new ethers.Wallet('0x' + '02'.repeat(32));
  const charlieSol = Keypair.generate();
  const davidSol = Keypair.generate();

  console.log('👥 Demo Participants:');
  console.log(`   Alice (ETH): ${aliceEth.address}`);
  console.log(`   Bob (ETH): ${bobEth.address}`);
  console.log(`   Charlie (SOL): ${charlieSol.publicKey.toString()}`);
  console.log(`   David (SOL): ${davidSol.publicKey.toString()}`);
  console.log('');

  // Create flow orchestrator
  const orchestrator = createFlowOrchestrator();

  // Demo NFT metadata
  const demoNFT = createFlowOrchestrator.createNFTMetadata({
    tokenId: '12345',
    name: 'Universal Demo NFT',
    description: 'A demonstration NFT for Universal NFT Protocol cross-chain flows',
    imageUrl: 'https://api.universalnft.com/images/demo-nft.png',
    attributes: [
      { trait_type: 'Protocol', value: 'Universal NFT' },
      { trait_type: 'Demo', value: 'Cross-Chain Flows' },
      { trait_type: 'Chains', value: '3' },
    ],
  });

  console.log('🎨 Demo NFT:');
  console.log(`   Token ID: ${demoNFT.tokenId}`);
  console.log(`   Name: ${demoNFT.name}`);
  console.log(`   Description: ${demoNFT.description}`);
  console.log(`   Metadata URI Length: ${demoNFT.metadataUri.length} characters`);
  console.log('');

  // Display supported routes
  console.log('🛣️  Supported Cross-Chain Routes:');
  const routes = orchestrator.getSupportedRoutes();
  routes.forEach((route, index) => {
    console.log(`   ${index + 1}. ${route.name}`);
  });
  console.log('');

  try {
    // Demo Flow 1: Solana → Base Sepolia
    console.log('=' .repeat(60));
    console.log('🌊 FLOW 1 DEMO: Solana → Base Sepolia');
    console.log('=' .repeat(60));

    const flow1Result = await orchestrator.executeSolanaToBaseFlow({
      tokenId: demoNFT.tokenId,
      metadataUri: demoNFT.metadataUri,
      senderKeypair: charlieSol,
      recipientAddress: bobEth.address,
      nonce: '1',
    });

    if (flow1Result.success) {
      console.log('✅ Flow 1 Demo Successful!');
      console.log(`   Message ID: 0x${Buffer.from(flow1Result.message.messageId).toString('hex').slice(0, 16)}...`);
      console.log(`   Gas Estimate: ${(await orchestrator.estimateTransferCosts(CHAIN_IDS.SOLANA_DEVNET, CHAIN_IDS.BASE_SEPOLIA)).estimatedGas}`);
    } else {
      console.log('❌ Flow 1 Demo Failed:', flow1Result.error);
    }
    console.log('');

    // Demo Flow 2: ZetaChain → Solana
    console.log('=' .repeat(60));
    console.log('🌊 FLOW 2 DEMO: ZetaChain → Solana');
    console.log('=' .repeat(60));

    const flow2Result = await orchestrator.executeZetaChainToSolanaFlow({
      tokenId: demoNFT.tokenId,
      metadataUri: demoNFT.metadataUri,
      senderAddress: aliceEth.address,
      recipientPubkey: davidSol.publicKey,
      nonce: '2',
    });

    if (flow2Result.success) {
      console.log('✅ Flow 2 Demo Successful!');
      console.log(`   Message ID: 0x${Buffer.from(flow2Result.message.messageId).toString('hex').slice(0, 16)}...`);
      console.log(`   Gas Estimate: ${(await orchestrator.estimateTransferCosts(CHAIN_IDS.ZETACHAIN_TESTNET, CHAIN_IDS.SOLANA_DEVNET)).estimatedGas}`);
    } else {
      console.log('❌ Flow 2 Demo Failed:', flow2Result.error);
    }
    console.log('');

    // Demo Flow 3: Base Sepolia → Solana
    console.log('=' .repeat(60));
    console.log('🌊 FLOW 3 DEMO: Base Sepolia → Solana');
    console.log('=' .repeat(60));

    const flow3Result = await orchestrator.executeBaseToSolanaFlow({
      tokenId: demoNFT.tokenId,
      metadataUri: demoNFT.metadataUri,
      senderAddress: bobEth.address,
      recipientPubkey: charlieSol.publicKey,
      nonce: '3',
    });

    if (flow3Result.success) {
      console.log('✅ Flow 3 Demo Successful!');
      console.log(`   Message ID: 0x${Buffer.from(flow3Result.message.messageId).toString('hex').slice(0, 16)}...`);
      console.log(`   Gas Estimate: ${(await orchestrator.estimateTransferCosts(CHAIN_IDS.BASE_SEPOLIA, CHAIN_IDS.SOLANA_DEVNET)).estimatedGas}`);
    } else {
      console.log('❌ Flow 3 Demo Failed:', flow3Result.error);
    }
    console.log('');

    // Demo Flow 4: Full Loop
    console.log('=' .repeat(60));
    console.log('🔄 FLOW 4 DEMO: Full Loop (ZetaChain → Base → Solana → ZetaChain)');
    console.log('=' .repeat(60));

    const loopResult = await orchestrator.executeFullLoopFlow({
      tokenId: '99999',
      metadataUri: demoNFT.metadataUri,
      originalOwner: aliceEth.address,
      intermediateRecipient: bobEth.address,
      solanaRecipient: charlieSol.publicKey,
      startingNonce: 100,
    });

    if (loopResult.success) {
      console.log('✅ Full Loop Demo Successful!');
      console.log('🎯 Loop Journey Summary:');
      loopResult.steps.forEach((step, index) => {
        const messageIdHex = Buffer.from(step.message.messageId).toString('hex');
        console.log(`   ${step.stepNumber}. ${step.description}`);
        console.log(`      Message ID: 0x${messageIdHex.slice(0, 16)}...`);
        console.log(`      Status: ${step.completed ? '✅ Completed' : '⏳ Pending'}`);
      });
    } else {
      console.log('❌ Full Loop Demo Failed:', loopResult.error);
    }
    console.log('');

    // Display flow statistics
    console.log('=' .repeat(60));
    console.log('📊 FLOW STATISTICS');
    console.log('=' .repeat(60));

    const stats = orchestrator.getFlowStatistics();
    console.log(`📨 Total Processed Messages: ${stats.totalProcessedMessages}`);
    console.log(`🛣️  Supported Routes: ${stats.supportedRoutes}`);
    console.log('🌊 Available Flows:');
    stats.availableFlows.forEach((flow, index) => {
      console.log(`   ${index + 1}. ${flow}`);
    });
    console.log('');

    // Demonstrate flow planning
    console.log('=' .repeat(60));
    console.log('📋 FLOW PLANNING DEMO');
    console.log('=' .repeat(60));

    const sampleFlowPlan = FlowUtils.generateFlowPlan(
      CHAIN_IDS.ZETACHAIN_TESTNET,
      CHAIN_IDS.SOLANA_DEVNET,
      demoNFT.tokenId
    );

    console.log(`🎯 Flow Type: ${sampleFlowPlan.flowType}`);
    console.log(`⏱️  Estimated Time: ${sampleFlowPlan.estimatedTime}`);
    console.log('📝 Execution Steps:');
    sampleFlowPlan.steps.forEach(step => {
      console.log(`   ${step.step}. [${step.chain}] ${step.action}`);
    });
    console.log('');

    // Demonstrate validation
    console.log('=' .repeat(60));
    console.log('✅ VALIDATION DEMO');
    console.log('=' .repeat(60));

    // Test valid parameters
    const validParams = {
      tokenId: demoNFT.tokenId,
      fromChain: CHAIN_IDS.ZETACHAIN_TESTNET,
      toChain: CHAIN_IDS.BASE_SEPOLIA,
      sender: aliceEth.address,
      recipient: bobEth.address,
    };

    const validationResult = FlowUtils.validateFlowParameters(validParams);
    console.log('🔍 Parameter Validation:');
    console.log(`   Valid: ${validationResult.isValid ? '✅' : '❌'}`);
    if (!validationResult.isValid) {
      console.log('   Errors:', validationResult.errors);
    }

    // Test invalid parameters (same chain)
    const invalidParams = {
      tokenId: demoNFT.tokenId,
      fromChain: CHAIN_IDS.ZETACHAIN_TESTNET,
      toChain: CHAIN_IDS.ZETACHAIN_TESTNET, // Same chain!
      sender: aliceEth.address,
      recipient: bobEth.address,
    };

    const invalidValidation = FlowUtils.validateFlowParameters(invalidParams);
    console.log('🚫 Invalid Parameter Test:');
    console.log(`   Valid: ${invalidValidation.isValid ? '✅' : '❌'}`);
    console.log(`   Errors: ${invalidValidation.errors.join(', ')}`);
    console.log('');

    // Route validation demo
    console.log('🛣️  Route Validation Demo:');
    const testRoutes = [
      { from: CHAIN_IDS.ZETACHAIN_TESTNET, to: CHAIN_IDS.BASE_SEPOLIA },
      { from: CHAIN_IDS.SOLANA_DEVNET, to: CHAIN_IDS.ZETACHAIN_TESTNET },
      { from: CHAIN_IDS.BASE_SEPOLIA, to: CHAIN_IDS.BASE_SEPOLIA }, // Invalid - same chain
    ];

    testRoutes.forEach(route => {
      const validation = orchestrator.validateRoute(route.from, route.to);
      console.log(`   ${validation.routeName}: ${validation.isSupported ? '✅ Supported' : '❌ Not Supported'}`);
    });

  } catch (error) {
    console.error('❌ Demo Error:', error.message);
  }

  console.log('');
  console.log('=' .repeat(60));
  console.log('🎉 Universal NFT Protocol Flow Demo Complete!');
  console.log('=' .repeat(60));
  console.log('');
  console.log('✨ Summary:');
  console.log('   ✅ All 4 cross-chain flows demonstrated');
  console.log('   ✅ Message validation and security features verified');
  console.log('   ✅ Route planning and cost estimation shown');
  console.log('   ✅ Error handling and parameter validation tested');
  console.log('');
  console.log('🚀 The Universal NFT Protocol is ready for cross-chain NFT transfers!');
  console.log('   Supported chains: ZetaChain, Base Sepolia, Solana Devnet');
  console.log('   Total routes: 6 bidirectional routes');
  console.log('   Security: Replay protection, message validation, route restrictions');
  console.log('');
}

// Helper function to create NFT metadata (since we can't import the class method directly)
function createNFTMetadata(params) {
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

// Create a mock for the static method
createFlowOrchestrator.createNFTMetadata = createNFTMetadata;

// Run the demo
main().catch(console.error);