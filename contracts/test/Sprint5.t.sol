// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {PolicyAdmin} from "../src/PolicyAdmin.sol";
import {PaymentToken} from "../src/PaymentToken.sol";
import {PaymasterEntry} from "../src/PaymasterEntry.sol";

contract PolicyPrecompileShim5 {
    address constant POLICY_ADMIN = address(uint160(0xF211));

    fallback(bytes calldata data) external returns (bytes memory) {
        (bool ok, bytes memory ret) = POLICY_ADMIN.staticcall(data);
        require(ok, "f211");
        return ret;
    }
}

contract IsoMemoPrecompileShim5 {
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

/// @notice Sprint 5: live fee on memo + two-owner dual-control (distinct addresses).
contract Sprint5Test is Test {
    address constant F201 = address(uint160(0xF201));
    address constant F202 = address(uint160(0xF202));
    address constant F210 = address(uint160(0xF210));
    address constant F211 = address(uint160(0xF211));
    address constant F213 = address(uint160(0xF213));

    address ownerA = address(0xA11CE);
    address ownerB = address(0xB0B);
    address alice = address(0xA71CE);
    address bob = address(0xB0B1);
    address feeRecipient = address(0xFEE0);

    PolicyAdmin policy;
    PaymentToken token;
    PaymasterEntry paymaster;

    function setUp() public {
        require(ownerA != ownerB, "owners must differ");

        deployCodeTo("PolicyAdmin.sol:PolicyAdmin", abi.encode(ownerA, ownerB), F211);
        policy = PolicyAdmin(F211);

        deployCodeTo("PaymentToken.sol:PaymentToken", abi.encode(alice, 1_000_000e6), F210);
        token = PaymentToken(F210);

        deployCodeTo("PaymasterEntry.sol:PaymasterEntry", abi.encode(ownerA, feeRecipient), F213);
        paymaster = PaymasterEntry(F213);

        PolicyPrecompileShim5 pshim = new PolicyPrecompileShim5();
        IsoMemoPrecompileShim5 ishim = new IsoMemoPrecompileShim5();
        vm.etch(F201, address(pshim).code);
        vm.etch(F202, address(ishim).code);

        vm.prank(ownerA);
        paymaster.setFeeTokenAllowlisted(F210, true);
    }

    function test_memo_takesFee_quoteMatches() public {
        uint256 fee = paymaster.quote(F210, token.FEE_GAS_LIMIT(), token.FEE_GAS_PRICE());
        assertEq(fee, token.FEE_GAS_LIMIT() * token.FEE_GAS_PRICE());

        uint256 aliceBefore = token.balanceOf(alice);
        uint256 recipBefore = token.balanceOf(feeRecipient);
        uint256 bobBefore = token.balanceOf(bob);
        uint256 principal = 100e6;

        vm.prank(alice, alice);
        token.transferWithMemo(
            bob,
            principal,
            keccak256("E2E-S5"),
            bytes16(keccak256("UETR-S5")),
            keccak256("INSTR-S5"),
            bytes3("USD"),
            0,
            bytes32(0),
            bytes32(0)
        );

        uint256 taken = token.balanceOf(feeRecipient) - recipBefore;
        assertEq(taken, fee);
        // quote == taken within 1 wei
        assertTrue(taken + 1 >= fee && fee + 1 >= taken);
        assertEq(token.balanceOf(bob), bobBefore + principal);
        assertEq(token.balanceOf(alice), aliceBefore - principal - fee);
    }

    function test_plainTransfer_isFeeFree() public {
        uint256 recipBefore = token.balanceOf(feeRecipient);
        vm.prank(alice);
        token.transfer(bob, 10e6);
        assertEq(token.balanceOf(feeRecipient), recipBefore);
        assertEq(token.balanceOf(bob), 10e6);
    }

    function test_policyReject_noFee_noPrincipal() public {
        vm.prank(ownerA);
        policy.freeze(alice);

        uint256 aliceBefore = token.balanceOf(alice);
        uint256 bobBefore = token.balanceOf(bob);
        uint256 recipBefore = token.balanceOf(feeRecipient);

        vm.prank(alice, alice);
        vm.expectRevert(abi.encodeWithSelector(PaymentToken.PolicyRejected.selector, uint16(1)));
        token.transferWithMemo(
            bob,
            100e6,
            keccak256("E2E-FRZ"),
            bytes16(keccak256("UETR-FRZ")),
            keccak256("INSTR-FRZ"),
            bytes3("USD"),
            0,
            bytes32(0),
            bytes32(0)
        );

        assertEq(token.balanceOf(alice), aliceBefore);
        assertEq(token.balanceOf(bob), bobBefore);
        assertEq(token.balanceOf(feeRecipient), recipBefore);
    }

    function test_eoa_takeFee_reverts() public {
        vm.expectRevert(PaymasterEntry.NotPaymentToken.selector);
        paymaster.takeFee(alice, F210, 21_000);
    }

    function test_freeze_fromA_immediate() public {
        vm.prank(ownerA);
        policy.freeze(alice);
        assertTrue(policy.frozen(alice));
    }

    function test_unfreeze_fromA_alone_reverts() public {
        vm.prank(ownerA);
        policy.freeze(alice);

        vm.prank(ownerA);
        policy.proposeUnfreeze(alice);

        // Same key confirm does not count.
        vm.prank(ownerA);
        vm.expectRevert(PolicyAdmin.SameOwnerConfirm.selector);
        policy.confirmUnfreeze(alice);

        assertTrue(policy.frozen(alice));
    }

    /// @dev Foundry two-owner unfreeze (propose A + confirm B).
    function test_unfreeze_proposeA_confirmB() public {
        vm.prank(ownerA);
        policy.freeze(alice);

        vm.prank(ownerA);
        policy.proposeUnfreeze(alice);

        vm.prank(ownerB);
        policy.confirmUnfreeze(alice);

        assertFalse(policy.frozen(alice));
    }

    function test_setThreshold_fromA_alone_doesNotStick() public {
        vm.prank(ownerA);
        policy.proposeSetThreshold(1_000e6);
        assertEq(policy.threshold(), 0);

        vm.prank(ownerA);
        vm.expectRevert(PolicyAdmin.SameOwnerConfirm.selector);
        policy.confirmSetThreshold();

        assertEq(policy.threshold(), 0);

        vm.prank(ownerB);
        policy.confirmSetThreshold();
        assertEq(policy.threshold(), 1_000e6);
    }
}
