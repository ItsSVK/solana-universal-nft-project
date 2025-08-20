const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying UniversalNFT to ZetaChain Athens Testnet...");

  // Get the contract factory
  const UniversalNFT = await ethers.getContractFactory("UniversalNFT");

  // Deploy the contract
  // Constructor parameters: name, symbol, gateway
  const gatewayAddress = "0x6c533f7fe93fae114d0954697069df33c9b74fd7"; // ZetaChain Athens testnet gateway
  
  const universalNFT = await UniversalNFT.deploy(
    "Universal NFT",
    "UNFT",
    gatewayAddress
  );

  await universalNFT.deployed();

  console.log("UniversalNFT deployed to:", universalNFT.address);
  console.log("Transaction hash:", universalNFT.deployTransaction.hash);

  // Wait for block confirmations
  await universalNFT.deployTransaction.wait(2);

  console.log("Deployment confirmed!");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });