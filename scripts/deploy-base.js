const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying UniversalNFTReceiver to Base Sepolia...");

  // Get the contract factory
  const UniversalNFTReceiver = await ethers.getContractFactory("UniversalNFTReceiver");

  // Deploy the contract
  // Constructor parameters: name, symbol, gateway
  const gatewayAddress = "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0"; // Base Sepolia gateway (placeholder)
  
  const nftReceiver = await UniversalNFTReceiver.deploy(
    "Universal NFT Receiver",
    "UNFTR",
    gatewayAddress
  );

  await nftReceiver.deployed();

  console.log("UniversalNFTReceiver deployed to:", nftReceiver.address);
  console.log("Transaction hash:", nftReceiver.deployTransaction.hash);

  // Wait for block confirmations
  await nftReceiver.deployTransaction.wait(2);

  console.log("Deployment confirmed!");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });