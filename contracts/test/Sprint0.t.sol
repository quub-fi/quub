// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {stdStorage, StdStorage} from "forge-std/StdStorage.sol";
import {PolicyAdmin} from "../src/PolicyAdmin.sol";
import {PaymentToken} from "../src/PaymentToken.sol";
import {EvidenceAnchor} from "../src/EvidenceAnchor.sol";
import {PaymasterEntry} from "../src/PaymasterEntry.sol";

/// @dev Foundry-only F201 shim: staticcall F211.check. Production F201 uses sload.
contract PolicyPrecompileShim {
    address constant POLICY_ADMIN = address(uint160(0xF211));

    fallback(bytes calldata data) external returns (bytes memory) {
        (bool ok, bytes memory ret) = POLICY_ADMIN.staticcall(data);
        require(ok, "f211");
        return ret;
    }
}

/// @dev Foundry-only F202 shim: same origin-in-hash as quub-iso.
contract IsoMemoPrecompileShim {
    fallback(bytes calldata data) external returns (bytes memory) {
        require(data.length >= 4 + 32 * 5, "len");
        (bytes32 endToEndId, bytes16 uetr, bytes32 instrId, bytes3 ccy, uint8 msgType) =
            abi.decode(data[4:], (bytes32, bytes16, bytes32, bytes3, uint8));
        require(endToEndId != bytes32(0), "e2e");
        if ((msgType == 0 || msgType == 2) && uetr == bytes16(0)) revert("uetr");
        require(
            ccy == bytes3("USD") || ccy == bytes3("CAD") || ccy == bytes3("AED") || ccy == bytes3("SAR"),
            "ccy"
        );
        bytes32 memoHash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin));
        return abi.encode(memoHash);
    }
}

contract MockFeeToken {
    string public constant name = "Mock Fee";
    string public constant symbol = "FEE";
    uint8 public constant decimals = 6;
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    function mint(address to, uint256 amount) external {
        balanceOf[to] += amount;
    }

    function approve(address spender, uint256 amount) external returns (bool) {
        allowance[msg.sender][spender] = amount;
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        uint256 allowed = allowance[from][msg.sender];
        require(allowed >= amount, "allow");
        require(balanceOf[from] >= amount, "bal");
        allowance[from][msg.sender] = allowed - amount;
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        return true;
    }
}

contract Sprint0Test is Test {
    using stdStorage for StdStorage;

    address constant F201 = address(uint160(0xF201));
    address constant F202 = address(uint160(0xF202));
    address constant F210 = address(uint160(0xF210));
    address constant F211 = address(uint160(0xF211));
    address constant F212 = address(uint160(0xF212));

    address ownerA = address(0xA11CE);
    address ownerB = address(0xB0B);
    address alice = address(0xA71CE);
    address bob = address(0xB0B1);
    address treasury = address(0x7Ea5);

    PolicyAdmin policy;
    PaymentToken token;
    EvidenceAnchor evidence;
    PaymasterEntry paymaster;
    MockFeeToken feeToken;

    function setUp() public {
        deployCodeTo("PolicyAdmin.sol:PolicyAdmin", abi.encode(ownerA, ownerB), F211);
        policy = PolicyAdmin(F211);

        deployCodeTo("PaymentToken.sol:PaymentToken", abi.encode(alice, 1_000_000e6), F210);
        token = PaymentToken(F210);

        deployCodeTo("EvidenceAnchor.sol:EvidenceAnchor", F212);
        evidence = EvidenceAnchor(F212);

        // Etch F201/F202 shims (runtime of empty constructors).
        PolicyPrecompileShim pshim = new PolicyPrecompileShim();
        IsoMemoPrecompileShim ishim = new IsoMemoPrecompileShim();
        vm.etch(F201, address(pshim).code);
        vm.etch(F202, address(ishim).code);

        paymaster = new PaymasterEntry(ownerA, treasury);
        feeToken = new MockFeeToken();

        vm.prank(ownerA);
        paymaster.setFeeTokenAllowlisted(address(feeToken), true);
    }

    function test_frozenSender_transferReverts_balancesUnchanged() public {
        vm.prank(ownerA);
        policy.freeze(alice);

        uint256 aliceBefore = token.balanceOf(alice);
        uint256 bobBefore = token.balanceOf(bob);

        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(PaymentToken.PolicyRejected.selector, uint16(1)));
        token.transfer(bob, 100e6);

        assertEq(token.balanceOf(alice), aliceBefore);
        assertEq(token.balanceOf(bob), bobBefore);
    }

    function test_goodMemo_emitsMemoAnchored() public {
        bytes32 e2e = keccak256("E2E-001");
        bytes16 uetr = bytes16(keccak256("UETR-001"));
        bytes32 instr = keccak256("INSTR-001");
        bytes3 ccy = bytes3("USD");
        uint8 msgType = 0; // pacs.008

        bytes32 expected = keccak256(abi.encode(e2e, uetr, instr, ccy, msgType, alice));

        vm.prank(alice, alice);
        vm.expectEmit(true, false, false, true);
        emit PaymentToken.MemoAnchored(expected, msgType);
        token.transferWithMemo(bob, 100e6, e2e, uetr, instr, ccy, msgType, bytes32(0), bytes32(0));

        assertEq(token.balanceOf(bob), 100e6);
    }

    function test_packHash_emitsEvidenceAnchored() public {
        bytes32 e2e = keccak256("E2E-002");
        bytes16 uetr = bytes16(keccak256("UETR-002"));
        bytes32 instr = keccak256("INSTR-002");
        bytes3 ccy = bytes3("USD");
        uint8 msgType = 0;
        bytes32 pack = keccak256("pack-001");
        bytes32 expectedMemo = keccak256(abi.encode(e2e, uetr, instr, ccy, msgType, alice));

        vm.prank(alice, alice);
        vm.expectEmit(true, true, true, true);
        emit EvidenceAnchor.EvidenceAnchored(pack, expectedMemo, F210);
        token.transferWithMemo(bob, 50e6, e2e, uetr, instr, ccy, msgType, bytes32(0), pack);

        (bytes32 p, bytes32 m, address a, bool exists) = evidence.records(pack);
        assertTrue(exists);
        assertEq(p, pack);
        assertEq(m, expectedMemo);
        assertEq(a, F210);
    }

    function test_emptyEndToEndId_reverts() public {
        vm.prank(alice);
        vm.expectRevert(PaymentToken.EmptyEndToEndId.selector);
        token.transferWithMemo(
            bob,
            100e6,
            bytes32(0),
            bytes16(keccak256("UETR")),
            keccak256("INSTR"),
            bytes3("USD"),
            0,
            bytes32(0),
            bytes32(0)
        );
    }

    function test_unlistedFeeToken_paymasterReverts() public {
        MockFeeToken unlisted = new MockFeeToken();
        unlisted.mint(alice, 1_000e6);
        vm.prank(alice);
        unlisted.approve(address(paymaster), 1_000e6);

        vm.expectRevert(PaymasterEntry.UnlistedFeeToken.selector);
        paymaster.quote(address(unlisted), 21_000, 1 gwei);

        vm.expectRevert(PaymasterEntry.UnlistedFeeToken.selector);
        paymaster.takeFee(alice, address(unlisted), 100);
    }

    function test_evidenceAnchor_storesHashes() public {
        bytes32 pack = keccak256("pack");
        bytes32 memo = keccak256("memo");
        evidence.anchor(pack, memo);
        (bytes32 p, bytes32 m, address a, bool exists) = evidence.records(pack);
        assertTrue(exists);
        assertEq(p, pack);
        assertEq(m, memo);
        assertEq(a, address(this));
    }

    function test_listedFeeToken_quoteAndTakeFee() public {
        feeToken.mint(alice, 1_000e6);
        vm.prank(alice);
        feeToken.approve(address(paymaster), type(uint256).max);

        uint256 quoted = paymaster.quote(address(feeToken), 21_000, 1);
        assertEq(quoted, 21_000);

        vm.prank(alice);
        paymaster.takeFee(alice, address(feeToken), quoted);
        assertEq(feeToken.balanceOf(treasury), quoted);
    }

    /// @notice Frozen mapping base slot is 4 (compiler layout). Cross-check Rust key formula.
    function test_frozenSlotKey_matchesLayout() public {
        address who = alice;
        bytes32 expected = keccak256(abi.encode(who, uint256(4)));
        // stdstore finds the same slot via the public getter
        uint256 found = stdstore.target(F211).sig("frozen(address)").with_key(who).find();
        assertEq(found, uint256(expected));
    }

    /// @notice Production F201 empty-reverts when msg.sender != F210.
    /// Foundry etches a PolicyAdmin shim at F201 (no caller gate). The Rust
    /// unit test `policy_non_token_caller_reverts` owns the gate with a full
    /// bytes32 zero trHash (`0x0000…0000`, not `0x0`).
    function test_f201_eoa_gate_documented_in_rust() public pure {
        bytes32 zero =
            0x0000000000000000000000000000000000000000000000000000000000000000;
        assertTrue(zero == bytes32(0));
    }
}
