// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.24;

/// @title PolicyAdmin — F211 facade (year-1 dual-control stub)
/// @notice freeze / unfreeze / pause / thresholds / fee token + propose/execute.
contract PolicyAdmin {
    address public ownerA;
    address public ownerB;
    bool public paused;
    uint256 public threshold;
    address public feeToken;

    mapping(address => bool) public frozen;
    mapping(bytes32 => Proposal) public proposals;

    struct Proposal {
        address target;
        bytes data;
        bool executed;
        bool approvedA;
        bool approvedB;
    }

    event Frozen(address indexed account);
    event Unfrozen(address indexed account);
    event Paused(bool paused);
    event ThresholdSet(uint256 threshold);
    event FeeTokenSet(address indexed feeToken);
    event Proposed(bytes32 indexed id, address indexed target);
    event Executed(bytes32 indexed id);

    error NotOwner();
    error AlreadyFrozen();
    error NotFrozen();
    error BadProposal();
    error DualControlIncomplete();
    error ZeroAddress();

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

    function unfreeze(address account) external onlyOwner {
        if (!frozen[account]) revert NotFrozen();
        frozen[account] = false;
        emit Unfrozen(account);
    }

    function pause() external onlyOwner {
        paused = true;
        emit Paused(true);
    }

    function unpause() external onlyOwner {
        paused = false;
        emit Paused(false);
    }

    function setThreshold(uint256 threshold_) external onlyOwner {
        threshold = threshold_;
        emit ThresholdSet(threshold_);
    }

    function setFeeToken(address feeToken_) external onlyOwner {
        if (feeToken_ == address(0)) revert ZeroAddress();
        feeToken = feeToken_;
        emit FeeTokenSet(feeToken_);
    }

    /// @notice Dual-control stub: either owner proposes; both must approve before execute.
    function propose(bytes32 id, address target, bytes calldata data) external onlyOwner {
        if (target == address(0) || id == bytes32(0)) revert BadProposal();
        Proposal storage p = proposals[id];
        if (p.target != address(0)) revert BadProposal();
        p.target = target;
        p.data = data;
        if (msg.sender == ownerA) p.approvedA = true;
        if (msg.sender == ownerB) p.approvedB = true;
        emit Proposed(id, target);
    }

    function approve(bytes32 id) external onlyOwner {
        Proposal storage p = proposals[id];
        if (p.target == address(0) || p.executed) revert BadProposal();
        if (msg.sender == ownerA) p.approvedA = true;
        if (msg.sender == ownerB) p.approvedB = true;
    }

    function execute(bytes32 id) external onlyOwner {
        Proposal storage p = proposals[id];
        if (p.target == address(0) || p.executed) revert BadProposal();
        if (!p.approvedA || !p.approvedB) revert DualControlIncomplete();
        p.executed = true;
        (bool ok,) = p.target.call(p.data);
        require(ok, "exec failed");
        emit Executed(id);
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
