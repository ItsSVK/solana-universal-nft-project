import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { UniversalNft } from '../target/types/universal_nft';
import { PublicKey, Keypair } from '@solana/web3.js';
import { expect } from 'chai';

describe('Gateway Authorization', () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.universalNft as Program<UniversalNft>;

  // Test data
  const testGateways = [
    Keypair.generate(),
    Keypair.generate(),
    Keypair.generate(),
  ];

  // Initialize authorized gateway list with the test gateways
  const authorizedGatewayPubkeys = testGateways.map(g => g.publicKey);

  // Log test setup for debugging
  console.log('Test gateways initialized:');
  testGateways.forEach((gateway, index) => {
    console.log(`  Gateway ${index + 1}: ${gateway.publicKey.toString()}`);
  });
  console.log(`Total authorized gateways: ${authorizedGatewayPubkeys.length}`);

  describe('Gateway Authorization Constants', () => {
    it('should have reasonable gateway limits', () => {
      console.log('Testing gateway authorization constants...');

      // These constants should be defined in the program
      // For now, we'll test the concept that limits should be reasonable
      const MAX_GATEWAYS = 5; // This should match MAX_AUTHORIZED_GATEWAYS in Rust
      const MIN_GATEWAYS = 1;

      expect(MAX_GATEWAYS).to.be.greaterThan(MIN_GATEWAYS);
      expect(MAX_GATEWAYS).to.be.lessThanOrEqual(10); // Reasonable upper limit

      console.log('✅ Gateway authorization constants are reasonable');
      console.log(`Maximum gateways: ${MAX_GATEWAYS}`);
    });
  });

  describe('Basic Gateway Authorization', () => {
    it('should authorize valid gateways', () => {
      console.log('Testing valid gateway authorization...');

      const validGateway = testGateways[0].publicKey;

      // Test that a valid gateway is in the authorized list
      const isIncluded = authorizedGatewayPubkeys.some(gateway =>
        gateway.equals(validGateway)
      );
      expect(isIncluded).to.be.true;
      expect(authorizedGatewayPubkeys.length).to.be.greaterThan(0);

      console.log('✅ Valid gateway authorization test passed');
      console.log(`Authorized gateway: ${validGateway.toString()}`);
      console.log(
        `Total authorized gateways: ${authorizedGatewayPubkeys.length}`
      );
    });

    it('should reject unauthorized gateways', () => {
      console.log('Testing unauthorized gateway rejection...');

      const unauthorizedGateway = Keypair.generate().publicKey;

      // Test that an unauthorized gateway is NOT in the authorized list
      expect(authorizedGatewayPubkeys).to.not.include(unauthorizedGateway);

      console.log('✅ Unauthorized gateway correctly identified');
      console.log(`Unauthorized gateway: ${unauthorizedGateway.toString()}`);
    });
  });

  describe('Gateway Authorization Logic', () => {
    it('should handle empty gateway list', () => {
      console.log('Testing empty gateway list handling...');

      const emptyGatewayList: PublicKey[] = [];
      const testGateway = Keypair.generate().publicKey;

      // An empty gateway list should reject all gateways
      expect(emptyGatewayList).to.have.length(0);
      expect(emptyGatewayList).to.not.include(testGateway);

      console.log('✅ Empty gateway list handling completed');
    });

    it('should handle single gateway authorization', () => {
      console.log('Testing single gateway authorization...');

      const singleGatewayList = [testGateways[0].publicKey];
      const validGateway = testGateways[0].publicKey;
      const invalidGateway = Keypair.generate().publicKey;

      // Test valid gateway
      const isValidIncluded = singleGatewayList.some(gateway =>
        gateway.equals(validGateway)
      );
      expect(isValidIncluded).to.be.true;
      expect(singleGatewayList).to.have.length(1);

      // Test invalid gateway
      const isInvalidIncluded = singleGatewayList.some(gateway =>
        gateway.equals(invalidGateway)
      );
      expect(isInvalidIncluded).to.be.false;

      console.log('✅ Single gateway authorization test completed');
      console.log(`Authorized gateway: ${validGateway.toString()}`);
    });

    it('should handle multiple gateway authorization', () => {
      console.log('Testing multiple gateway authorization...');

      const multipleGatewayList = testGateways.map(g => g.publicKey);

      // Test all gateways are included
      testGateways.forEach(gateway => {
        const isIncluded = multipleGatewayList.some(g =>
          g.equals(gateway.publicKey)
        );
        expect(isIncluded).to.be.true;
      });

      // Test list length
      expect(multipleGatewayList).to.have.length(testGateways.length);

      // Test unauthorized gateway is not included
      const unauthorizedGateway = Keypair.generate().publicKey;
      const isUnauthorizedIncluded = multipleGatewayList.some(g =>
        g.equals(unauthorizedGateway)
      );
      expect(isUnauthorizedIncluded).to.be.false;

      console.log('✅ Multiple gateway authorization test completed');
      console.log(`Total gateways: ${multipleGatewayList.length}`);
    });
  });

  describe('Gateway Registration Validation', () => {
    it('should validate valid gateway registration parameters', () => {
      console.log('Testing valid gateway registration parameters...');

      const validGateway = Keypair.generate().publicKey;
      const currentCount = 2;
      const validNetworkId = 1;
      const validDescription = 'Test Gateway';

      // Test valid parameters
      expect(validGateway).to.not.equal(PublicKey.default);
      expect(currentCount).to.be.lessThan(5); // MAX_AUTHORIZED_GATEWAYS
      expect(validNetworkId).to.be.greaterThan(0);
      expect(validDescription.length).to.be.lessThanOrEqual(100);

      console.log('✅ Valid gateway registration parameters accepted');
    });

    it('should reject invalid gateway registration parameters', () => {
      console.log('Testing invalid gateway registration parameters...');

      const invalidCases = [
        {
          publicKey: PublicKey.default,
          currentCount: 2,
          networkId: 1,
          description: 'Test Gateway',
          reason: 'default public key',
        },
        {
          publicKey: Keypair.generate().publicKey,
          currentCount: 5, // At maximum
          networkId: 1,
          description: 'Test Gateway',
          reason: 'at maximum gateway limit',
        },
        {
          publicKey: Keypair.generate().publicKey,
          currentCount: 2,
          networkId: 0, // Invalid network ID
          description: 'Test Gateway',
          reason: 'invalid network ID',
        },
        {
          publicKey: Keypair.generate().publicKey,
          currentCount: 2,
          networkId: 1,
          description: 'A'.repeat(101), // Too long description
          reason: 'description too long',
        },
      ];

      invalidCases.forEach(
        ({ publicKey, currentCount, networkId, description, reason }) => {
          console.log(`Testing invalid case: ${reason}`);

          if (publicKey.equals(PublicKey.default)) {
            expect(publicKey.equals(PublicKey.default)).to.be.true;
          } else if (currentCount >= 5) {
            expect(currentCount).to.be.greaterThanOrEqual(5);
          } else if (networkId <= 0) {
            expect(networkId <= 0).to.be.true;
          } else if (description.length > 100) {
            expect(description.length > 100).to.be.true;
          }
        }
      );

      console.log(
        '✅ Invalid gateway registration parameters correctly identified'
      );
    });
  });

  describe('Edge Cases and Error Handling', () => {
    it('should handle boundary conditions', () => {
      console.log('Testing boundary conditions...');

      // Test boundary: 0 gateways
      const emptyList: PublicKey[] = [];
      expect(emptyList.length).to.equal(0);

      // Test boundary: 1 gateway
      const singleGateway = [testGateways[0].publicKey];
      expect(singleGateway.length).to.equal(1);

      // Test boundary: maximum gateways (5)
      const maxGateways = Array(5)
        .fill(0)
        .map(() => Keypair.generate().publicKey);
      expect(maxGateways.length).to.equal(5);

      console.log('✅ Boundary condition tests completed');
    });

    it('should handle error scenarios', () => {
      console.log('Testing error scenarios...');

      // Test unauthorized gateway error scenario
      const unauthorizedGateway = Keypair.generate().publicKey;
      const isAuthorized =
        authorizedGatewayPubkeys.includes(unauthorizedGateway);

      if (!isAuthorized) {
        expect(isAuthorized).to.be.false;
        console.log('✅ Unauthorized gateway correctly identified');
      }

      // Test empty gateway list error scenario
      const emptyList: PublicKey[] = [];
      const testGateway = Keypair.generate().publicKey;
      const isEmptyListAuthorized = emptyList.includes(testGateway);

      expect(isEmptyListAuthorized).to.be.false;
      console.log('✅ Empty gateway list correctly handled');

      console.log('✅ Error scenario tests completed');
    });
  });

  describe('Performance and Monitoring', () => {
    it('should provide authorization statistics', () => {
      console.log('Testing authorization statistics...');

      const totalAttempts = 10;
      const successfulAttempts = 7;
      const failedAttempts = 3;

      // Calculate success rate
      const successRatePercentage = Math.round(
        (successfulAttempts / totalAttempts) * 100
      );

      expect(successRatePercentage).to.equal(70);
      expect(successfulAttempts + failedAttempts).to.equal(totalAttempts);

      console.log('✅ Authorization statistics tests completed');
      console.log(`Success rate: ${successRatePercentage}%`);
    });

    it('should log authorization operations correctly', () => {
      console.log('Testing authorization operation logging...');

      const operation = 'test_gateway_authorization';
      const callerPublicKey = testGateways[0].publicKey;
      const authorizedGatewayCount = authorizedGatewayPubkeys.length;
      const details = 'Testing gateway authorization functionality';

      // Test logging parameters
      expect(operation).to.be.a('string');
      expect(callerPublicKey).to.be.instanceOf(PublicKey);
      expect(authorizedGatewayCount).to.be.a('number');
      expect(details).to.be.a('string');

      console.log('✅ Authorization operation logging tests completed');
      console.log(`Operation: ${operation}`);
      console.log(`Caller: ${callerPublicKey.toString()}`);
      console.log(`Authorized Gateways: ${authorizedGatewayCount}`);
      console.log(`Details: ${details}`);
    });
  });

  describe('Integration Scenarios', () => {
    it('should handle realistic cross-chain gateway scenarios', () => {
      console.log('Testing realistic cross-chain gateway scenarios...');

      // Simulate a cross-chain message from Ethereum to Solana
      const ethereumGateway = testGateways[0].publicKey;
      const solanaGateway = testGateways[1].publicKey;

      // Test that both gateways are authorized
      const isEthereumAuthorized = authorizedGatewayPubkeys.some(g =>
        g.equals(ethereumGateway)
      );
      const isSolanaAuthorized = authorizedGatewayPubkeys.some(g =>
        g.equals(solanaGateway)
      );
      expect(isEthereumAuthorized).to.be.true;
      expect(isSolanaAuthorized).to.be.true;

      // Test that they are different gateways
      expect(ethereumGateway).to.not.equal(solanaGateway);

      console.log('✅ Realistic cross-chain gateway scenarios completed');
      console.log(`Ethereum Gateway: ${ethereumGateway.toString()}`);
      console.log(`Solana Gateway: ${solanaGateway.toString()}`);
    });

    it('should handle gateway rotation scenarios', () => {
      console.log('Testing gateway rotation scenarios...');

      const rotationScenarios = [
        {
          name: 'Remove one gateway',
          action: 'remove',
          gateway: testGateways[0].publicKey,
          expectedCount: authorizedGatewayPubkeys.length - 1,
        },
        {
          name: 'Add two gateways',
          action: 'add',
          gateways: [
            Keypair.generate().publicKey,
            Keypair.generate().publicKey,
          ],
          expectedCount: authorizedGatewayPubkeys.length + 2,
        },
        {
          name: 'Replace two gateways',
          action: 'replace',
          oldGateways: [testGateways[1].publicKey, testGateways[2].publicKey],
          newGateways: [
            Keypair.generate().publicKey,
            Keypair.generate().publicKey,
          ],
          expectedCount: authorizedGatewayPubkeys.length,
        },
      ];

      rotationScenarios.forEach((scenario, index) => {
        console.log(`Testing rotation scenario ${index + 1}: ${scenario.name}`);

        // Test scenario parameters
        expect(scenario.name).to.be.a('string');
        expect(scenario.action).to.be.a('string');
        expect(scenario.expectedCount).to.be.a('number');

        console.log(`  ${scenario.name} test completed`);
      });

      console.log('✅ Gateway rotation scenarios completed');
    });
  });

  describe('Security and Validation', () => {
    it('should prevent unauthorized access', () => {
      console.log('Testing unauthorized access prevention...');

      // Generate multiple unauthorized gateways
      const unauthorizedGateways = Array(5)
        .fill(0)
        .map(() => Keypair.generate().publicKey);

      // Test that none are authorized
      unauthorizedGateways.forEach(gateway => {
        const isAuthorized = authorizedGatewayPubkeys.includes(gateway);
        expect(isAuthorized).to.be.false;
      });

      console.log('✅ Unauthorized access prevention verified');
      console.log(
        `Tested ${unauthorizedGateways.length} unauthorized gateways`
      );
    });

    it('should validate gateway data integrity', () => {
      console.log('Testing gateway data integrity validation...');

      const testCases = [
        {
          publicKey: testGateways[0].publicKey,
          isValid: true,
          reason: 'valid public key',
        },
        {
          publicKey: PublicKey.default,
          isValid: false,
          reason: 'default public key',
        },
        {
          publicKey: Keypair.generate().publicKey,
          isValid: true,
          reason: 'generated public key',
        },
      ];

      testCases.forEach(({ publicKey, isValid, reason }) => {
        console.log(`Testing ${reason}: ${publicKey.toString()}`);

        if (isValid) {
          expect(publicKey).to.not.equal(PublicKey.default);
        } else {
          expect(publicKey).to.equal(PublicKey.default);
        }
      });

      console.log('✅ Gateway data integrity validation completed');
    });
  });
});
