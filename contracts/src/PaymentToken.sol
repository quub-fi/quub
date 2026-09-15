// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.24;

/// @title PaymentToken — F210 ERC-20 with transferWithMemo
/// @notice Calls F201 then F202 (and F212 if packHash != 0). No native / gas token.
/// @dev Runtime has no immutables so genesis etch at 0x…F210 is legal.
contract PaymentToken {
    address constant QUUB_POLICY = address(uint160(0xF201));
    address constant QUUB_ISO_MEMO = address(uint160(0xF202));
    address constant EVIDENCE_ANCHOR = address(uint160(0xF212));

    string public constant name = "Quub Payment Token";
    string public constant symbol = "QPT";
    uint8 public constant decimals = 6;

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
    error PolicyCallFailed();
    error MemoCallFailed();

    constructor(address mintTo, uint256 supply) {
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
    /// @param packHash If non-zero, anchors (packHash, memoHash) at F212.
    function transferWithMemo(
        address to,
        uint256 amount,
        bytes32 endToEndId,
        bytes16 uetr,
        bytes32 instrId,
        bytes3 ccy,
        uint8 msgType,
        bytes32 trHash,
        bytes32 packHash
    ) external returns (bytes32 memoHash) {
        _requireMemoFields(endToEndId, uetr, ccy, msgType);
        _policy(msg.sender, to, amount, trHash);
        _move(msg.sender, to, amount);
        memoHash = _commitMemo(endToEndId, uetr, instrId, ccy, msgType);
        emit MemoAnchored(memoHash, msgType);
        if (packHash != bytes32(0)) {
            (bool ok,) = EVIDENCE_ANCHOR.call(
                abi.encodeWithSignature("anchor(bytes32,bytes32)", packHash, memoHash)
            );
            require(ok, "anchor failed");
        }
    }

    function _policy(address from, address to, uint256 amount, bytes32 trHash) internal view {
        (bool ok, bytes memory ret) = QUUB_POLICY.staticcall(
            abi.encodeWithSignature(
                "check(address,address,address,uint256,bytes32)",
                address(this),
                from,
                to,
                amount,
                trHash
            )
        );
        if (!ok || ret.length < 64) revert PolicyCallFailed();
        (bool allowed, uint16 reason) = abi.decode(ret, (bool, uint16));
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

    function _requireMemoFields(bytes32 endToEndId, bytes16 uetr, bytes3 ccy, uint8 msgType)
        internal
        pure
    {
        if (endToEndId == bytes32(0)) revert EmptyEndToEndId();
        if ((msgType == 0 || msgType == 2) && uetr == bytes16(0)) revert MissingUetr();
        if (!_allowedCcy(ccy)) revert BadCurrency();
    }

    /// @dev Hash comes only from F202 — no local keccak that can drift.
    function _commitMemo(
        bytes32 endToEndId,
        bytes16 uetr,
        bytes32 instrId,
        bytes3 ccy,
        uint8 msgType
    ) internal view returns (bytes32 memoHash) {
        (bool ok, bytes memory ret) = QUUB_ISO_MEMO.staticcall(
            abi.encodeWithSignature(
                "validateAndCommit(bytes32,bytes16,bytes32,bytes3,uint8)",
                endToEndId,
                uetr,
                instrId,
                ccy,
                msgType
            )
        );
        if (!ok || ret.length < 32) revert MemoCallFailed();
        memoHash = abi.decode(ret, (bytes32));
    }

    function _allowedCcy(bytes3 ccy) internal pure returns (bool) {
        return ccy == bytes3("USD") || ccy == bytes3("CAD") || ccy == bytes3("AED")
            || ccy == bytes3("SAR");
    }
}
