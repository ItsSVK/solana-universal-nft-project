#!/usr/bin/env node

/**
 * Comprehensive End-to-End Test Runner for Universal NFT Protocol
 * Executes all test suites with proper setup, reporting, and cleanup
 */

const { exec, spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');

class E2ETestRunner {
  constructor() {
    this.results = {
      startTime: Date.now(),
      suites: [],
      summary: {
        totalSuites: 0,
        passedSuites: 0,
        failedSuites: 0,
        totalTests: 0,
        passedTests: 0,
        failedTests: 0,
        skippedTests: 0,
        totalDuration: 0,
      }
    };
    
    this.testSuites = [
      {
        name: 'Environment Setup',
        file: 'test/utils/TestEnvironment.ts',
        description: 'Basic environment and utility tests',
        timeout: 60000,
        required: true,
      },
      {
        name: 'Enhanced End-to-End Integration',
        file: 'test/EndToEndEnhanced.test.ts', 
        description: 'Comprehensive cross-chain NFT lifecycle tests',
        timeout: 300000, // 5 minutes
        required: true,
      },
      {
        name: 'Performance & Load Testing',
        file: 'test/PerformanceAndLoad.test.ts',
        description: 'Performance benchmarks and load testing',
        timeout: 600000, // 10 minutes
        required: false,
      },
      {
        name: 'Security & Robustness Testing',
        file: 'test/SecurityAndRobustness.test.ts',
        description: 'Security attack vectors and robustness scenarios',
        timeout: 300000, // 5 minutes
        required: false,
      },
      {
        name: 'Original End-to-End Integration',
        file: 'test/EndToEndIntegration.test.ts',
        description: 'Original comprehensive integration tests',
        timeout: 300000, // 5 minutes
        required: false,
      },
    ];
  }

  async checkPrerequisites() {
    console.log('🔍 Checking prerequisites...');
    
    try {
      // Check if Hardhat is available
      await this.executeCommand('npx hardhat --version');
      console.log('   ✅ Hardhat available');
      
      // Check if contracts are compiled
      try {
        await fs.access('artifacts');
        console.log('   ✅ Contract artifacts found');
      } catch {
        console.log('   📋 Compiling contracts...');
        await this.executeCommand('npm run build:shared');
        await this.executeCommand('npm run build:solidity');
        console.log('   ✅ Contracts compiled');
      }
      
      // Check test files exist
      let missingFiles = [];
      for (const suite of this.testSuites) {
        try {
          await fs.access(suite.file);
        } catch {
          missingFiles.push(suite.file);
        }
      }
      
      if (missingFiles.length > 0) {
        throw new Error(`Missing test files: ${missingFiles.join(', ')}`);
      }
      console.log('   ✅ All test files found');
      
      console.log('✅ Prerequisites check passed');
      return true;
      
    } catch (error) {
      console.error('❌ Prerequisites check failed:', error.message);
      return false;
    }
  }

  async executeCommand(command, options = {}) {
    return new Promise((resolve, reject) => {
      exec(command, options, (error, stdout, stderr) => {
        if (error) {
          reject(error);
        } else {
          resolve({ stdout, stderr });
        }
      });
    });
  }

  async runTestSuite(suite) {
    console.log(`\n${'='.repeat(80)}`);
    console.log(`🧪 Running: ${suite.name}`);
    console.log(`📋 Description: ${suite.description}`);
    console.log(`⏱️  Timeout: ${suite.timeout / 1000}s`);
    console.log(`${'='.repeat(80)}`);
    
    const startTime = Date.now();
    const suiteResult = {
      name: suite.name,
      file: suite.file,
      description: suite.description,
      startTime,
      endTime: null,
      duration: 0,
      success: false,
      required: suite.required,
      output: '',
      error: null,
      tests: {
        total: 0,
        passed: 0,
        failed: 0,
        skipped: 0,
      }
    };

    try {
      // Use Mocha directly with TypeScript support
      const mochaCommand = `npx mocha --require ts-node/register --timeout ${suite.timeout} "${suite.file}" --reporter json`;
      
      const { stdout, stderr } = await this.executeCommand(mochaCommand, {
        maxBuffer: 1024 * 1024 * 10, // 10MB buffer for large outputs
      });

      suiteResult.endTime = Date.now();
      suiteResult.duration = suiteResult.endTime - startTime;
      suiteResult.output = stdout;

      try {
        // Parse Mocha JSON output
        const mochaResults = JSON.parse(stdout);
        suiteResult.tests.total = mochaResults.stats.tests || 0;
        suiteResult.tests.passed = mochaResults.stats.passes || 0;
        suiteResult.tests.failed = mochaResults.stats.failures || 0;
        suiteResult.tests.skipped = mochaResults.stats.pending || 0;
        suiteResult.success = mochaResults.stats.failures === 0;
      } catch (parseError) {
        // If JSON parsing fails, check for general success indicators
        suiteResult.success = !stderr && stdout.includes('passing');
        console.log('   ⚠️  Could not parse detailed test results, using general success indicator');
      }

      if (suiteResult.success) {
        console.log(`✅ ${suite.name} PASSED`);
        console.log(`   ⏱️  Duration: ${(suiteResult.duration / 1000).toFixed(2)}s`);
        console.log(`   📊 Tests: ${suiteResult.tests.passed} passed, ${suiteResult.tests.failed} failed, ${suiteResult.tests.skipped} skipped`);
      } else {
        console.log(`❌ ${suite.name} FAILED`);
        console.log(`   ⏱️  Duration: ${(suiteResult.duration / 1000).toFixed(2)}s`);
        console.log(`   📊 Tests: ${suiteResult.tests.passed} passed, ${suiteResult.tests.failed} failed, ${suiteResult.tests.skipped} skipped`);
        if (stderr) {
          console.log(`   🔍 Error output: ${stderr.slice(0, 500)}...`);
        }
      }

    } catch (error) {
      suiteResult.endTime = Date.now();
      suiteResult.duration = suiteResult.endTime - startTime;
      suiteResult.success = false;
      suiteResult.error = error.message;

      console.log(`❌ ${suite.name} FAILED WITH ERROR`);
      console.log(`   ⏱️  Duration: ${(suiteResult.duration / 1000).toFixed(2)}s`);
      console.log(`   ❌ Error: ${error.message.slice(0, 200)}...`);

      if (suite.required) {
        console.log(`   🚨 This is a REQUIRED test suite - considering stopping execution`);
      }
    }

    this.results.suites.push(suiteResult);
    return suiteResult;
  }

  async generateReport() {
    const endTime = Date.now();
    this.results.summary.totalDuration = endTime - this.results.startTime;

    // Calculate summary statistics
    this.results.summary.totalSuites = this.results.suites.length;
    this.results.summary.passedSuites = this.results.suites.filter(s => s.success).length;
    this.results.summary.failedSuites = this.results.suites.filter(s => !s.success).length;
    
    this.results.summary.totalTests = this.results.suites.reduce((sum, s) => sum + s.tests.total, 0);
    this.results.summary.passedTests = this.results.suites.reduce((sum, s) => sum + s.tests.passed, 0);
    this.results.summary.failedTests = this.results.suites.reduce((sum, s) => sum + s.tests.failed, 0);
    this.results.summary.skippedTests = this.results.suites.reduce((sum, s) => sum + s.tests.skipped, 0);

    // Generate detailed report
    const report = `
# Universal NFT Protocol - End-to-End Test Report

**Generated:** ${new Date().toISOString()}
**Total Duration:** ${(this.results.summary.totalDuration / 1000 / 60).toFixed(2)} minutes

## Executive Summary

- **Test Suites:** ${this.results.summary.passedSuites}/${this.results.summary.totalSuites} passed
- **Individual Tests:** ${this.results.summary.passedTests}/${this.results.summary.totalTests} passed
- **Success Rate:** ${((this.results.summary.passedTests / Math.max(this.results.summary.totalTests, 1)) * 100).toFixed(1)}%

## Test Suite Results

${this.results.suites.map(suite => `
### ${suite.name} ${suite.success ? '✅' : '❌'}

- **File:** \`${suite.file}\`
- **Description:** ${suite.description}
- **Duration:** ${(suite.duration / 1000).toFixed(2)}s
- **Required:** ${suite.required ? 'Yes' : 'No'}
- **Tests:** ${suite.tests.passed} passed, ${suite.tests.failed} failed, ${suite.tests.skipped} skipped

${suite.error ? `**Error:** ${suite.error}` : ''}
`).join('')}

## Performance Metrics

${this.results.suites.map(suite => `
- **${suite.name}:** ${(suite.duration / 1000).toFixed(2)}s`).join('')}

## Recommendations

${this.generateRecommendations()}

## Raw Results

\`\`\`json
${JSON.stringify(this.results, null, 2)}
\`\`\`
`;

    // Write report to file
    const reportPath = 'test-results/e2e-test-report.md';
    try {
      await fs.mkdir('test-results', { recursive: true });
      await fs.writeFile(reportPath, report);
      console.log(`📄 Detailed report written to: ${reportPath}`);
    } catch (error) {
      console.log(`⚠️  Could not write report file: ${error.message}`);
    }

    return report;
  }

  generateRecommendations() {
    const recommendations = [];
    
    const failedRequired = this.results.suites.filter(s => s.required && !s.success);
    const failedOptional = this.results.suites.filter(s => !s.required && !s.success);
    
    if (failedRequired.length > 0) {
      recommendations.push('🚨 **CRITICAL:** Required test suites failed - protocol not ready for production');
      recommendations.push('   - Review and fix issues in: ' + failedRequired.map(s => s.name).join(', '));
    }
    
    if (failedOptional.length > 0) {
      recommendations.push('⚠️  **WARNING:** Optional test suites failed - consider addressing');
      recommendations.push('   - Review issues in: ' + failedOptional.map(s => s.name).join(', '));
    }
    
    const successRate = (this.results.summary.passedTests / Math.max(this.results.summary.totalTests, 1)) * 100;
    
    if (successRate >= 95) {
      recommendations.push('✅ **EXCELLENT:** Protocol shows exceptional quality and readiness');
    } else if (successRate >= 90) {
      recommendations.push('✅ **GOOD:** Protocol shows good quality with minor issues');
    } else if (successRate >= 75) {
      recommendations.push('⚠️  **ACCEPTABLE:** Protocol quality acceptable but needs improvement');
    } else {
      recommendations.push('❌ **POOR:** Protocol needs significant work before production');
    }
    
    // Performance recommendations
    const avgDuration = this.results.summary.totalDuration / Math.max(this.results.suites.length, 1);
    if (avgDuration > 60000) { // More than 1 minute average
      recommendations.push('⚡ **PERFORMANCE:** Consider optimizing test performance');
    }
    
    return recommendations.join('\n');
  }

  async run() {
    console.log('🚀 Starting Universal NFT Protocol End-to-End Test Suite');
    console.log(`📅 Started at: ${new Date().toISOString()}`);
    console.log(`🧪 Test suites to run: ${this.testSuites.length}`);
    
    // Check prerequisites
    if (!await this.checkPrerequisites()) {
      console.log('❌ Prerequisites failed - aborting test run');
      process.exit(1);
    }

    let continueExecution = true;

    // Run each test suite
    for (const [index, suite] of this.testSuites.entries()) {
      if (!continueExecution) {
        console.log('🛑 Stopping execution due to critical failures');
        break;
      }

      console.log(`\n📊 Progress: ${index + 1}/${this.testSuites.length} test suites`);
      
      const result = await this.runTestSuite(suite);
      
      // Stop execution if required suite fails
      if (suite.required && !result.success) {
        console.log(`🚨 Required test suite '${suite.name}' failed`);
        console.log('🤔 Continue with remaining tests? (y/N)');
        
        // For automated runs, we'll continue, but mark as problematic
        console.log('🤖 Automated run - continuing with remaining tests');
        console.log('⚠️  Production readiness is QUESTIONABLE due to required test failure');
      }
    }

    // Generate final report
    console.log('\n' + '='.repeat(100));
    console.log('📊 GENERATING FINAL REPORT');
    console.log('='.repeat(100));

    await this.generateReport();

    // Print summary
    console.log('\n🏁 UNIVERSAL NFT PROTOCOL END-TO-END TEST SUMMARY');
    console.log('='.repeat(80));
    console.log(`⏱️  Total Duration: ${(this.results.summary.totalDuration / 1000 / 60).toFixed(2)} minutes`);
    console.log(`📊 Test Suites: ${this.results.summary.passedSuites}/${this.results.summary.totalSuites} passed`);
    console.log(`📋 Individual Tests: ${this.results.summary.passedTests}/${this.results.summary.totalTests} passed`);
    console.log(`📈 Success Rate: ${((this.results.summary.passedTests / Math.max(this.results.summary.totalTests, 1)) * 100).toFixed(1)}%`);

    // Final verdict
    const requiredFailed = this.results.suites.filter(s => s.required && !s.success).length > 0;
    const overallSuccess = this.results.summary.passedSuites === this.results.summary.totalSuites;

    console.log('\n🎯 FINAL VERDICT:');
    if (overallSuccess) {
      console.log('✅ ALL TESTS PASSED - PROTOCOL READY FOR PRODUCTION! 🚀');
    } else if (!requiredFailed) {
      console.log('⚠️  MOSTLY SUCCESSFUL - Protocol ready with minor issues to address');
    } else {
      console.log('❌ CRITICAL ISSUES - Protocol NOT ready for production');
    }

    console.log('\n📄 Detailed report available in: test-results/e2e-test-report.md');
    console.log('🎉 End-to-end testing completed!');

    // Exit with appropriate code
    process.exit(requiredFailed ? 1 : 0);
  }
}

// Handle CLI arguments
const args = process.argv.slice(2);
const flags = {
  skipOptional: args.includes('--skip-optional'),
  verbose: args.includes('--verbose'),
  performance: args.includes('--performance-only'),
  security: args.includes('--security-only'),
  help: args.includes('--help'),
};

if (flags.help) {
  console.log(`
Universal NFT Protocol E2E Test Runner

Usage: node scripts/run-e2e-tests.js [options]

Options:
  --skip-optional      Skip optional test suites (performance, security)
  --performance-only   Run only performance tests
  --security-only      Run only security tests  
  --verbose           Enable verbose output
  --help              Show this help message

Examples:
  node scripts/run-e2e-tests.js                    # Run all tests
  node scripts/run-e2e-tests.js --skip-optional    # Run only required tests
  node scripts/run-e2e-tests.js --performance-only # Run only performance tests
`);
  process.exit(0);
}

// Create and run test runner
const runner = new E2ETestRunner();

// Filter test suites based on flags
if (flags.skipOptional) {
  runner.testSuites = runner.testSuites.filter(suite => suite.required);
}

if (flags.performanceOnly) {
  runner.testSuites = runner.testSuites.filter(suite => 
    suite.name.includes('Performance') || suite.name.includes('Load')
  );
}

if (flags.securityOnly) {
  runner.testSuites = runner.testSuites.filter(suite => 
    suite.name.includes('Security') || suite.name.includes('Robustness')
  );
}

// Handle process termination gracefully
process.on('SIGINT', async () => {
  console.log('\n🛑 Received SIGINT - generating partial report...');
  await runner.generateReport();
  process.exit(130);
});

process.on('SIGTERM', async () => {
  console.log('\n🛑 Received SIGTERM - generating partial report...');
  await runner.generateReport();
  process.exit(143);
});

// Start the test runner
runner.run().catch(error => {
  console.error('💥 Fatal error in test runner:', error);
  process.exit(1);
});