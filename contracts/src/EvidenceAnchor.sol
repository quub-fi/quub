// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.24;

/// @title EvidenceAnchor — F212 on-chain hash anchor (no PII)
contract EvidenceAnchor {
    struct Record {
        bytes32 packHash;
        bytes32 memoHash;
        address anchorer;
        bool exists;
    }

    mapping(bytes32 => Record) public records;

    event EvidenceAnchored(bytes32 indexed packHash, bytes32 indexed memoHash, address indexed anchorer);

    error ZeroHash();
    error AlreadyAnchored();

    function anchor(bytes32 packHash, bytes32 memoHash) external {
        if (packHash == bytes32(0) || memoHash == bytes32(0)) revert ZeroHash();
        if (records[packHash].exists) revert AlreadyAnchored();
        records[packHash] =
            Record({packHash: packHash, memoHash: memoHash, anchorer: msg.sender, exists: true});
        emit EvidenceAnchored(packHash, memoHash, msg.sender);
    }
}
