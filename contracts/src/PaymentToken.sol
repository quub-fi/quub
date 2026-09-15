// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.24;

import {PolicyAdmin} from "./PolicyAdmin.sol";

/// @title PaymentToken — F210 ERC-20 with transferWithMemo
/// @notice Calls PolicyAdmin.check before moving balances; emits MemoAnchored.
/// @dev This is an allowlisted payment stable facade, not a native / gas token.
contract PaymentToken {
    string public constant name = "Quub Payment Token";
    string public constant symbol = "QPT";
    uint8 public constant decimals = 6;

    PolicyAdmin public immutable policyAdmin;
    uint256 public totalSupply;
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event MemoAnchored(bytes32 indexed memoHash, uint8 msgType);

    error PolicyRejected(uint16 reason);
    error InsufficientBalance();
    error InsufficientAllowance();
    error EmptyEndToEndId();
    error MissingUetr();
    error BadCurrency();

    constructor(PolicyAdmin policyAdmin_, address mintTo, uint256 supply) {
        policyAdmin = policyAdmin_;
        balanceOf[mintTo] = supply;
        totalSupply = supply;
        emit Transfer(address(0), mintTo, supply);
    }

    function approve(address spender, uint256 amount) external returns (bool) {
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }

    function transfer(address to, uint256 amount) external returns (bool) {
        _policy(msg.sender, to, amount, bytes32(0));
        _move(msg.sender, to, amount);
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        uint256 allowed = allowance[from][msg.sender];
        if (allowed < amount) revert InsufficientAllowance();
        if (allowed != type(uint256).max) {
            allowance[from][msg.sender] = allowed - amount;
        }
        _policy(from, to, amount, bytes32(0));
        _move(from, to, amount);
        return true;
    }

    /// @notice Transfer with ISO memo identity set (no PII / no timestamp in hash).
    function transferWithMemo(
        address to,
        uint256 amount,
        bytes32 endToEndId,
        bytes16 uetr,
        bytes32 instrId,
        bytes3 ccy,
        uint8 msgType,
        bytes32 trHash
    ) external returns (bytes32 memoHash) {
        memoHash = _validateMemo(endToEndId, uetr, instrId, ccy, msgType);
        _policy(msg.sender, to, amount, trHash);
        _move(msg.sender, to, amount);
        emit MemoAnchored(memoHash, msgType);
    }

    function _policy(address from, address to, uint256 amount, bytes32 trHash) internal view {
        (bool allowed, uint16 reason) =
            policyAdmin.check(address(this), from, to, amount, trHash);
        if (!allowed) revert PolicyRejected(reason);
    }

    function _move(address from, address to, uint256 amount) internal {
        uint256 bal = balanceOf[from];
        if (bal < amount) revert InsufficientBalance();
        unchecked {
            balanceOf[from] = bal - amount;
            balanceOf[to] += amount;
        }
        emit Transfer(from, to, amount);
    }

    function _validateMemo(
        bytes32 endToEndId,
        bytes16 uetr,
        bytes32 instrId,
        bytes3 ccy,
        uint8 msgType
    ) internal view returns (bytes32 memoHash) {
        if (endToEndId == bytes32(0)) revert EmptyEndToEndId();
        // pacs.008 (0) and pacs.009 (2) require UETR
        if ((msgType == 0 || msgType == 2) && uetr == bytes16(0)) revert MissingUetr();
        if (!_allowedCcy(ccy)) revert BadCurrency();
        // No block.timestamp in the hash (AGENTS.md §4 F202).
        memoHash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin));
    }

    function _allowedCcy(bytes3 ccy) internal pure returns (bool) {
        return ccy == bytes3("USD") || ccy == bytes3("CAD") || ccy == bytes3("AED")
            || ccy == bytes3("SAR");
    }
}
