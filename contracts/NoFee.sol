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


contract InvoicePayment {
    address public tokenAddress;

    event PayInvoiceEvent(
        string invoice_id,
        address indexed seller,
        address indexed payer,
        uint128 paid_at,
        uint128 amount
    );

    constructor(address _tokenAddress) {
        tokenAddress = _tokenAddress;
    }

    function payInvoice(address seller, string memory invoice_id, uint256 amount) external {
        IERC20 token = IERC20(tokenAddress);

        uint256 allowedAmount = token.allowance(msg.sender, address(this));
        require(allowedAmount >= amount, "Not enough allowance for transfer");

        bool success = token.transferFrom(msg.sender, seller, amount);
        require(success, "Transfer failed");

        emit PayInvoiceEvent(invoice_id, seller, msg.sender, uint128(block.timestamp), uint128(amount));
    }
}
