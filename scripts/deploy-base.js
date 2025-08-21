const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying UniversalNFTReceiver to Base Sepolia...");

  // Get the contract factory
  const UniversalNFTReceiver = await ethers.getContractFactory("UniversalNFTReceiver");

  // Deploy the contract
  // Constructor parameters: gatewayAddress, initialOwner, zetaChainUniversalNFT
  const gatewayAddress = "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0"; // Base Sepolia gateway (placeholder)
  const [deployer] = await ethers.getSigners();
  const initialOwner = deployer.address;
  const zetaChainUniversalNFT = "0x0000000000000000000000000000000000000000"; // To be set after ZetaChain deployment
  
  console.log("Deploying with:", {
    gatewayAddress,
    initialOwner,
    zetaChainUniversalNFT
  });
  
  const nftReceiver = await UniversalNFTReceiver.deploy(
    gatewayAddress,
    initialOwner,
    zetaChainUniversalNFT
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