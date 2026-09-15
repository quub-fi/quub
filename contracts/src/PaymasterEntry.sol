// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.24;

/// @title Minimal fee-token surface used by F213 takeFee.
interface IPaymentTokenFee {
    function paymasterDebit(address from, address to, uint256 amount) external;
}

/// @title PaymasterEntry — F213 quote / takeFee wrappers
/// @notice Year-1 rate is a fixed 1e6-scale constant, not an oracle call.
/// @dev `feeRecipient` occupies the same storage slot formerly named `treasury`.
contract PaymasterEntry {
    /// @dev tokenAmount = gasLimit * gasPrice * RATE / 1e6 (RATE = 1e6 → 1:1 wei-scale stub)
    uint256 public constant RATE_1E6 = 1_000_000;

    address public constant PAYMENT_TOKEN = address(uint160(0xF210));

    address public owner;
    /// @dev Slot 1 — formerly `treasury`. Do not reshuffle.
    address public feeRecipient;
    mapping(address => bool) public feeTokenAllowlisted;

    event FeeTaken(address indexed payer, address indexed feeToken, uint256 tokenAmount);
    event FeeTokenAllowlisted(address indexed feeToken, bool allowed);

    error NotOwner();
    error NotPaymentToken();
    error UnlistedFeeToken();
    error ZeroAmount();

    modifier onlyOwner() {
        if (msg.sender != owner) revert NotOwner();
        _;
    }

    constructor(address owner_, address feeRecipient_) {
        owner = owner_;
        feeRecipient = feeRecipient_;
    }

    function setFeeTokenAllowlisted(address feeToken, bool allowed) external onlyOwner {
        feeTokenAllowlisted[feeToken] = allowed;
        emit FeeTokenAllowlisted(feeToken, allowed);
    }

    function quote(address feeToken, uint256 gasLimit, uint256 gasPrice)
        external
        view
        returns (uint256 tokenAmount)
    {
        if (!feeTokenAllowlisted[feeToken]) revert UnlistedFeeToken();
        tokenAmount = (gasLimit * gasPrice * RATE_1E6) / 1_000_000;
    }

    /// @notice Only F210 may call. Debits via `paymasterDebit` (no open transferFrom).
    function takeFee(address payer, address feeToken, uint256 tokenAmount) external {
        if (msg.sender != PAYMENT_TOKEN) revert NotPaymentToken();
        if (!feeTokenAllowlisted[feeToken]) revert UnlistedFeeToken();
        if (tokenAmount == 0) revert ZeroAmount();
        IPaymentTokenFee(feeToken).paymasterDebit(payer, feeRecipient, tokenAmount);
        emit FeeTaken(payer, feeToken, tokenAmount);
    }
}
