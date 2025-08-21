/**
 * Deployment script for Error Recovery and Monitoring system
 * Deploys all components needed for comprehensive error handling
 */

const { ethers } = require("hardhat");
const fs = require('fs').promises;
const path = require('path');

async function main() {
    console.log("🚀 Deploying Universal NFT Error Recovery System...");
    console.log("=" + "=".repeat(60));

    // Get deployer account
    const [deployer] = await ethers.getSigners();
    console.log("📋 Deploying contracts with account:", deployer.address);
    console.log("💰 Account balance:", ethers.formatEther(await deployer.provider.getBalance(deployer.address)), "ETH");

    const deploymentData = {
        network: hre.network.name,
        deployer: deployer.address,
        timestamp: new Date().toISOString(),
        contracts: {}
    };

    try {
        // 1. Deploy Monitoring Logger
        console.log("\n📊 Deploying MonitoringLogger...");
        const MonitoringLogger = await ethers.getContractFactory("MonitoringLogger");
        const monitoringLogger = await MonitoringLogger.deploy(deployer.address);
        await monitoringLogger.waitForDeployment();
        
        const monitoringLoggerAddress = await monitoringLogger.getAddress();
        console.log("   ✅ MonitoringLogger deployed to:", monitoringLoggerAddress);
        
        deploymentData.contracts.monitoringLogger = {
            address: monitoringLoggerAddress,
            deploymentHash: monitoringLogger.deploymentTransaction().hash
        };

        // 2. Deploy Error Recovery Manager
        console.log("\n🔧 Deploying ErrorRecoveryManager...");
        const ErrorRecoveryManager = await ethers.getContractFactory("ErrorRecoveryManager");
        const errorRecoveryManager = await ErrorRecoveryManager.deploy(deployer.address);
        await errorRecoveryManager.waitForDeployment();
        
        const errorRecoveryManagerAddress = await errorRecoveryManager.getAddress();
        console.log("   ✅ ErrorRecoveryManager deployed to:", errorRecoveryManagerAddress);
        
        deploymentData.contracts.errorRecoveryManager = {
            address: errorRecoveryManagerAddress,
            deploymentHash: errorRecoveryManager.deploymentTransaction().hash
        };

        // 3. Get existing contracts (if any)
        let gatewayAddress = "0x0000000000000000000000000000000000000000"; // Placeholder
        let existingUniversalNFT = null;

        try {
            const deploymentPath = path.join(__dirname, '..', 'deployments', `${hre.network.name}.json`);
            const existingDeployments = JSON.parse(await fs.readFile(deploymentPath, 'utf8'));
            if (existingDeployments.zetaChainNFT) {
                gatewayAddress = existingDeployments.gateway || gatewayAddress;
                existingUniversalNFT = existingDeployments.zetaChainNFT;
                console.log("   📋 Found existing UniversalNFT at:", existingUniversalNFT);
            }
        } catch (error) {
            console.log("   ⚠️  No existing deployments found, will deploy new contracts");
        }

        // 4. Deploy Enhanced Universal NFT (if gateway available)
        if (gatewayAddress !== "0x0000000000000000000000000000000000000000") {
            console.log("\n🎨 Deploying UniversalNFTEnhanced...");
            const UniversalNFTEnhanced = await ethers.getContractFactory("UniversalNFTEnhanced");
            const universalNFTEnhanced = await UniversalNFTEnhanced.deploy(
                gatewayAddress,
                deployer.address,
                errorRecoveryManagerAddress
            );
            await universalNFTEnhanced.waitForDeployment();
            
            const universalNFTEnhancedAddress = await universalNFTEnhanced.getAddress();
            console.log("   ✅ UniversalNFTEnhanced deployed to:", universalNFTEnhancedAddress);
            
            deploymentData.contracts.universalNFTEnhanced = {
                address: universalNFTEnhancedAddress,
                deploymentHash: universalNFTEnhanced.deploymentTransaction().hash
            };
        } else {
            console.log("   ⚠️  Skipping UniversalNFTEnhanced deployment (no gateway address)");
        }

        // 5. Setup roles and permissions
        console.log("\n🔐 Setting up roles and permissions...");
        
        // Grant roles on MonitoringLogger
        const LOGGER_ROLE = await monitoringLogger.LOGGER_ROLE();
        const MONITOR_ROLE = await monitoringLogger.MONITOR_ROLE();
        const AUDITOR_ROLE = await monitoringLogger.AUDITOR_ROLE();

        console.log("   📝 Granting MonitoringLogger roles...");
        await monitoringLogger.grantRole(LOGGER_ROLE, errorRecoveryManagerAddress);
        await monitoringLogger.grantRole(MONITOR_ROLE, deployer.address);
        await monitoringLogger.grantRole(AUDITOR_ROLE, deployer.address);
        console.log("   ✅ MonitoringLogger roles configured");

        // Grant roles on ErrorRecoveryManager
        const ADMIN_ROLE = await errorRecoveryManager.ADMIN_ROLE();
        const OPERATOR_ROLE = await errorRecoveryManager.OPERATOR_ROLE();
        const RECOVERY_ROLE = await errorRecoveryManager.RECOVERY_ROLE();

        console.log("   📝 Granting ErrorRecoveryManager roles...");
        await errorRecoveryManager.grantRole(ADMIN_ROLE, deployer.address);
        await errorRecoveryManager.grantRole(OPERATOR_ROLE, deployer.address);
        await errorRecoveryManager.grantRole(RECOVERY_ROLE, deployer.address);
        console.log("   ✅ ErrorRecoveryManager roles configured");

        // 6. Configure default settings
        console.log("\n⚙️  Configuring default settings...");
        
        // Set up default alert rules
        try {
            await errorRecoveryManager.createAlertRule(
                "High Error Rate",
                "operations_failed",
                10, // Threshold: 10 failed operations
                3600, // Window: 1 hour
                1800, // Cooldown: 30 minutes
                "Error rate is unusually high - manual review required"
            );
            console.log("   ✅ Default alert rules created");
        } catch (error) {
            console.log("   ⚠️  Alert rule creation failed:", error.message);
        }

        // Configure retry settings
        try {
            await errorRecoveryManager.updateRetryConfig(
                5,       // maxRetries
                300,     // baseDelay (5 minutes)
                14400,   // maxDelay (4 hours)
                2,       // backoffMultiplier
                true     // useExponentialBackoff
            );
            console.log("   ✅ Retry configuration set");
        } catch (error) {
            console.log("   ⚠️  Retry configuration failed:", error.message);
        }

        // 7. Initialize monitoring
        console.log("\n📊 Initializing monitoring system...");
        
        // Add initial log entry
        try {
            await monitoringLogger.addSimpleLog(
                1, // LogLevel.INFO
                9, // OperationType.SYSTEM
                "Deployment",
                "Error recovery system deployed and initialized"
            );
            console.log("   ✅ Initial monitoring log created");
        } catch (error) {
            console.log("   ⚠️  Initial logging failed:", error.message);
        }

        // 8. Verify deployment
        console.log("\n✅ Verifying deployment...");
        
        // Verify MonitoringLogger
        const loggingEnabled = await monitoringLogger.loggingEnabled();
        const metricsEnabled = await monitoringLogger.metricsEnabled();
        console.log(`   📊 MonitoringLogger: Logging=${loggingEnabled}, Metrics=${metricsEnabled}`);

        // Verify ErrorRecoveryManager
        const autoRetryEnabled = await errorRecoveryManager.autoRetryEnabled();
        const adminRecoveryEnabled = await errorRecoveryManager.adminRecoveryEnabled();
        console.log(`   🔧 ErrorRecoveryManager: AutoRetry=${autoRetryEnabled}, AdminRecovery=${adminRecoveryEnabled}`);

        // 9. Save deployment data
        console.log("\n💾 Saving deployment data...");
        
        const deploymentsDir = path.join(__dirname, '..', 'deployments');
        await fs.mkdir(deploymentsDir, { recursive: true });
        
        const deploymentFile = path.join(deploymentsDir, `error-recovery-${hre.network.name}.json`);
        await fs.writeFile(deploymentFile, JSON.stringify(deploymentData, null, 2));
        console.log("   ✅ Deployment data saved to:", deploymentFile);

        // 10. Generate configuration file
        const configData = {
            errorRecovery: {
                manager: errorRecoveryManagerAddress,
                monitoringLogger: monitoringLoggerAddress,
                autoRetryInterval: 3600, // 1 hour
                maxRetries: 5,
                baseRetryDelay: 300, // 5 minutes
                maxRetryDelay: 14400, // 4 hours
            },
            monitoring: {
                enabled: true,
                logLevel: "INFO",
                metricsEnabled: true,
                alertingEnabled: true,
            },
            contracts: deploymentData.contracts
        };

        const configFile = path.join(__dirname, '..', 'config', `error-recovery-${hre.network.name}.json`);
        await fs.mkdir(path.dirname(configFile), { recursive: true });
        await fs.writeFile(configFile, JSON.stringify(configData, null, 2));
        console.log("   ✅ Configuration file generated:", configFile);

        // 11. Summary
        console.log("\n🎉 DEPLOYMENT COMPLETE!");
        console.log("=" + "=".repeat(60));
        console.log("📋 Deployed Contracts:");
        console.log(`   🔧 ErrorRecoveryManager: ${errorRecoveryManagerAddress}`);
        console.log(`   📊 MonitoringLogger: ${monitoringLoggerAddress}`);
        if (deploymentData.contracts.universalNFTEnhanced) {
            console.log(`   🎨 UniversalNFTEnhanced: ${deploymentData.contracts.universalNFTEnhanced.address}`);
        }
        
        console.log("\n🔧 Next Steps:");
        console.log("   1. Update existing contracts to use ErrorRecoveryManager");
        console.log("   2. Configure monitoring dashboards");
        console.log("   3. Set up alerting endpoints");
        console.log("   4. Test error recovery scenarios");
        console.log("   5. Document operational procedures");

        console.log("\n📚 Documentation:");
        console.log("   • Error Recovery: See contracts/shared/ErrorRecoveryManager.sol");
        console.log("   • Monitoring: See contracts/shared/MonitoringLogger.sol");
        console.log("   • Configuration: See config/error-recovery-" + hre.network.name + ".json");

        return {
            errorRecoveryManager: errorRecoveryManagerAddress,
            monitoringLogger: monitoringLoggerAddress,
            universalNFTEnhanced: deploymentData.contracts.universalNFTEnhanced?.address
        };

    } catch (error) {
        console.error("\n❌ Deployment failed:", error);
        
        // Save partial deployment data for debugging
        if (Object.keys(deploymentData.contracts).length > 0) {
            const failedDeploymentFile = path.join(__dirname, '..', 'deployments', `failed-error-recovery-${hre.network.name}.json`);
            await fs.writeFile(failedDeploymentFile, JSON.stringify(deploymentData, null, 2));
            console.log("💾 Partial deployment data saved to:", failedDeploymentFile);
        }
        
        throw error;
    }
}

// Helper function to wait for transaction confirmations
async function waitForConfirmations(tx, confirmations = 2) {
    console.log(`   ⏳ Waiting for ${confirmations} confirmations...`);
    await tx.wait(confirmations);
    console.log("   ✅ Confirmed");
}

// Helper function to verify contract deployment
async function verifyContract(address, constructorArgs = []) {
    if (hre.network.name === "hardhat" || hre.network.name === "localhost") {
        console.log("   ⏭️  Skipping verification on local network");
        return;
    }

    try {
        console.log(`   🔍 Verifying contract at ${address}...`);
        await hre.run("verify:verify", {
            address: address,
            constructorArguments: constructorArgs,
        });
        console.log("   ✅ Contract verified");
    } catch (error) {
        console.log("   ⚠️  Verification failed:", error.message);
    }
}

// Script execution
if (require.main === module) {
    main()
        .then((addresses) => {
            console.log("\n🎯 Deployment addresses:");
            console.log(JSON.stringify(addresses, null, 2));
            process.exit(0);
        })
        .catch((error) => {
            console.error("💥 Fatal error:", error);
            process.exit(1);
        });
}

module.exports = { main };