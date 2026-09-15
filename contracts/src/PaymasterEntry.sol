// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.24;

/// @title Minimal ERC-20 used only as an allowlisted fee token in tests.
interface IERC20Lite {
    function balanceOf(address account) external view returns (uint256);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
}

/// @title PaymasterEntry — F213 quote / takeFee wrappers
/// @notice Year-1 rate is a fixed 1e6-scale constant, not an oracle call.
contract PaymasterEntry {
    /// @dev tokenAmount = gasLimit * gasPrice * RATE / 1e6 (RATE = 1 → 1:1 wei-scale stub)
    uint256 public constant RATE_1E6 = 1_000_000;

    address public owner;
    address public treasury;
    mapping(address => bool) public feeTokenAllowlisted;

    event FeeTaken(address indexed payer, address indexed feeToken, uint256 tokenAmount);
    event FeeTokenAllowlisted(address indexed feeToken, bool allowed);

    error NotOwner();
    error UnlistedFeeToken();
    error ZeroAmount();
    error TransferFailed();

    modifier onlyOwner() {
        if (msg.sender != owner) revert NotOwner();
        _;
    }

    constructor(address owner_, address treasury_) {
        owner = owner_;
        treasury = treasury_;
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

    function takeFee(address payer, address feeToken, uint256 tokenAmount) external {
        if (!feeTokenAllowlisted[feeToken]) revert UnlistedFeeToken();
        if (tokenAmount == 0) revert ZeroAmount();
        bool ok = IERC20Lite(feeToken).transferFrom(payer, treasury, tokenAmount);
        if (!ok) revert TransferFailed();
        emit FeeTaken(payer, feeToken, tokenAmount);
    }
}
