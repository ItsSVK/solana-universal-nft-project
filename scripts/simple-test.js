/**
 * Simple test to identify the issue with MonitoringLogger
 */

const { ethers } = require("hardhat");

async function main() {
    console.log("🧪 Simple Error Recovery Test...");

    const [deployer] = await ethers.getSigners();
    console.log("📋 Testing with account:", deployer.address);

    try {
        // Test 1: Deploy ErrorRecoveryManager only
        console.log("\n1️⃣ Testing ErrorRecoveryManager...");
        const ErrorRecoveryManager = await ethers.getContractFactory("ErrorRecoveryManager");
        const errorRecoveryManager = await ErrorRecoveryManager.deploy(deployer.address);
        await errorRecoveryManager.waitForDeployment();
        console.log("   ✅ ErrorRecoveryManager deployed successfully");

        // Test 2: Deploy MonitoringLogger with debug
        console.log("\n2️⃣ Testing MonitoringLogger deployment...");
        const MonitoringLogger = await ethers.getContractFactory("MonitoringLogger");
        
        console.log("   📋 Constructor args: owner =", deployer.address);
        const monitoringLogger = await MonitoringLogger.deploy(deployer.address);
        await monitoringLogger.waitForDeployment();
        console.log("   ✅ MonitoringLogger deployed successfully");

        // Test 3: Check initial state
        console.log("\n3️⃣ Testing MonitoringLogger initial state...");
        const loggingEnabled = await monitoringLogger.loggingEnabled();
        const metricsEnabled = await monitoringLogger.metricsEnabled();
        console.log("   📊 Logging enabled:", loggingEnabled);
        console.log("   📊 Metrics enabled:", metricsEnabled);

        // Test 4: Check roles
        console.log("\n4️⃣ Testing role management...");
        const LOGGER_ROLE = await monitoringLogger.LOGGER_ROLE();
        const hasLoggerRole = await monitoringLogger.hasRole(LOGGER_ROLE, deployer.address);
        console.log("   🔑 Deployer has logger role initially:", hasLoggerRole);

        // Grant logger role
        await monitoringLogger.grantRole(LOGGER_ROLE, deployer.address);
        const hasLoggerRoleAfter = await monitoringLogger.hasRole(LOGGER_ROLE, deployer.address);
        console.log("   🔑 Deployer has logger role after grant:", hasLoggerRoleAfter);

        // Test 5: Try logging (this was failing before)
        console.log("\n5️⃣ Testing logging functionality...");
        console.log("   📝 Attempting to add simple log...");
        
        // Check parameters before calling
        console.log("   📋 Log level: 1 (INFO)");
        console.log("   📋 Operation type: 9 (SYSTEM)");
        console.log("   📋 Component: 'Test'");
        console.log("   📋 Message: 'Simple test log'");

        const tx = await monitoringLogger.addSimpleLog(
            1, // INFO level
            9, // SYSTEM operation
            "Test",
            "Simple test log"
        );
        await tx.wait();
        console.log("   ✅ Simple log added successfully!");

        console.log("\n🎉 All tests passed!");
        return true;

    } catch (error) {
        console.error("\n❌ Test failed at:", error.message);
        console.error("Full error:", error);
        return false;
    }
}

main()
    .then((success) => {
        if (success) {
            console.log("\n✅ Test completed successfully");
            process.exit(0);
        } else {
            console.log("\n❌ Test failed");
            process.exit(1);
        }
    })
    .catch((error) => {
        console.error("💥 Fatal error:", error);
        process.exit(1);
    });