// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.24;

/// @title PolicyAdmin — F211 facade (maker-checker stub, ADR-019)
/// @notice freeze is immediate from either owner. unfreeze / setFeeToken / setThreshold / pause
///         require propose + confirm from the *other* owner. Same key twice does not count.
contract PolicyAdmin {
    address public ownerA;
    address public ownerB;
    bool public paused;
    uint256 public threshold;
    address public feeToken;

    mapping(address => bool) public frozen;

    /// @dev account => proposer (zero = no pending unfreeze)
    mapping(address => address) public pendingUnfreeze;
    address public pendingFeeToken;
    address public pendingFeeTokenProposer;
    uint256 public pendingThreshold;
    address public pendingThresholdProposer;
    bool public pendingPauseValue;
    address public pendingPauseProposer;
    bool public pendingPauseActive;

    event Frozen(address indexed account);
    event Unfrozen(address indexed account);
    event Paused(bool paused);
    event ThresholdSet(uint256 threshold);
    event FeeTokenSet(address indexed feeToken);
    event UnfreezeProposed(address indexed account, address indexed proposer);
    event FeeTokenProposed(address indexed feeToken, address indexed proposer);
    event ThresholdProposed(uint256 threshold, address indexed proposer);
    event PauseProposed(bool paused, address indexed proposer);

    error NotOwner();
    error AlreadyFrozen();
    error NotFrozen();
    error ZeroAddress();
    error NotProposed();
    error SameOwnerConfirm();
    error NotOtherOwner();

    modifier onlyOwner() {
        if (msg.sender != ownerA && msg.sender != ownerB) revert NotOwner();
        _;
    }

    constructor(address ownerA_, address ownerB_) {
        if (ownerA_ == address(0) || ownerB_ == address(0)) revert ZeroAddress();
        ownerA = ownerA_;
        ownerB = ownerB_;
    }

    function freeze(address account) external onlyOwner {
        if (account == address(0)) revert ZeroAddress();
        if (frozen[account]) revert AlreadyFrozen();
        frozen[account] = true;
        emit Frozen(account);
    }

    function proposeUnfreeze(address account) external onlyOwner {
        if (!frozen[account]) revert NotFrozen();
        pendingUnfreeze[account] = msg.sender;
        emit UnfreezeProposed(account, msg.sender);
    }

    function confirmUnfreeze(address account) external onlyOwner {
        address proposer = pendingUnfreeze[account];
        if (proposer == address(0)) revert NotProposed();
        _requireOtherOwner(proposer);
        delete pendingUnfreeze[account];
        frozen[account] = false;
        emit Unfrozen(account);
    }

    function proposePause(bool paused_) external onlyOwner {
        pendingPauseValue = paused_;
        pendingPauseProposer = msg.sender;
        pendingPauseActive = true;
        emit PauseProposed(paused_, msg.sender);
    }

    function confirmPause() external onlyOwner {
        if (!pendingPauseActive) revert NotProposed();
        _requireOtherOwner(pendingPauseProposer);
        paused = pendingPauseValue;
        pendingPauseActive = false;
        pendingPauseProposer = address(0);
        emit Paused(paused);
    }

    function proposeSetThreshold(uint256 threshold_) external onlyOwner {
        pendingThreshold = threshold_;
        pendingThresholdProposer = msg.sender;
        emit ThresholdProposed(threshold_, msg.sender);
    }

    function confirmSetThreshold() external onlyOwner {
        if (pendingThresholdProposer == address(0)) revert NotProposed();
        _requireOtherOwner(pendingThresholdProposer);
        threshold = pendingThreshold;
        pendingThresholdProposer = address(0);
        emit ThresholdSet(threshold);
    }

    function proposeSetFeeToken(address feeToken_) external onlyOwner {
        if (feeToken_ == address(0)) revert ZeroAddress();
        pendingFeeToken = feeToken_;
        pendingFeeTokenProposer = msg.sender;
        emit FeeTokenProposed(feeToken_, msg.sender);
    }

    function confirmSetFeeToken() external onlyOwner {
        if (pendingFeeTokenProposer == address(0)) revert NotProposed();
        _requireOtherOwner(pendingFeeTokenProposer);
        feeToken = pendingFeeToken;
        pendingFeeTokenProposer = address(0);
        emit FeeTokenSet(feeToken);
    }

    function _requireOtherOwner(address proposer) internal view {
        if (msg.sender == proposer) revert SameOwnerConfirm();
        // Confirm must be the other owner key (OwnerA propose → OwnerB confirm, or reverse).
        if (proposer == ownerA && msg.sender != ownerB) revert NotOtherOwner();
        if (proposer == ownerB && msg.sender != ownerA) revert NotOtherOwner();
        if (proposer != ownerA && proposer != ownerB) revert NotOtherOwner();
    }

    /// @notice View used by PaymentToken / F201 path. Fail closed.
    function check(
        address /* token */,
        address from,
        address to,
        uint256 amount,
        bytes32 /* trHash */
    ) external view returns (bool allowed, uint16 reason) {
        if (from == address(0) || to == address(0) || amount == 0) {
            return (false, 8); // malformed
        }
        if (paused) {
            return (false, 7);
        }
        if (frozen[from]) {
            return (false, 1);
        }
        if (frozen[to]) {
            return (false, 2);
        }
        if (threshold != 0 && amount > threshold) {
            return (false, 4);
        }
        return (true, 0);
    }
}
