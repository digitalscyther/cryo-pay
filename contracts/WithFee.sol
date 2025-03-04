// SPDX-License-Identifier: MIT

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

pragma solidity >=0.8.2 <0.9.0;


contract MyToken is ERC20 {
    constructor(uint256 initialSupply) ERC20("MyToken", "MTK") {
        _mint(msg.sender, initialSupply);
    }

    function decimals() public pure override returns (uint8) {
        return 6;
    }
}


contract InvoicePayment is Ownable {
    address public tokenAddress;

    event PayInvoiceEvent(
        string invoice_id,
        address indexed seller,
        address indexed payer,
        uint128 paid_at,
        uint128 amount
    );

    constructor(address _tokenAddress) Ownable(msg.sender) {
        tokenAddress = _tokenAddress;
    }

    function payInvoice(address seller, string memory invoice_id, uint256 amount) external {
        IERC20 token = IERC20(tokenAddress);

        uint256 allowedAmount = token.allowance(msg.sender, address(this));
        require(allowedAmount >= amount, "Not enough allowance for transfer");

        uint256 fee = (amount * 5) / 1000; // 0.5% fee
        uint256 amountAfterFee = amount - fee;

        bool success = token.transferFrom(msg.sender, seller, amountAfterFee);
        require(success, "Transfer to seller failed");

        success = token.transferFrom(msg.sender, address(this), fee);
        require(success, "Transfer of fee failed");

        emit PayInvoiceEvent(invoice_id, seller, msg.sender, uint128(block.timestamp), uint128(amountAfterFee));
    }

    function extractFees() external onlyOwner {
        IERC20 token = IERC20(tokenAddress);
        uint256 balance = token.balanceOf(address(this));
        require(balance > 0, "No fees to extract");

        bool success = token.transfer(owner(), balance);
        require(success, "Fee extraction failed");
    }
}
