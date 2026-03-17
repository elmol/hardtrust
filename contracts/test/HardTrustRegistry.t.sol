// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {HardTrustRegistry} from "../src/HardTrustRegistry.sol";

contract HardTrustRegistryTest is Test {
    HardTrustRegistry registry;

    address deployer = address(0x1);
    address attesterAddr = address(0x2);
    address randomAddr = address(0x3);

    bytes32 serialHash = keccak256("TEST-SERIAL-001");
    address deviceAddr = address(0xDEAD);

    function setUp() public {
        vm.prank(deployer);
        registry = new HardTrustRegistry(attesterAddr);
    }

    function test_attesterCanRegisterDevice() public {
        vm.prank(attesterAddr);
        registry.registerDevice(serialHash, deviceAddr);

        (address dAddr, address att, uint256 ts, bool active) = registry.getDevice(serialHash);
        assertEq(dAddr, deviceAddr);
        assertEq(att, attesterAddr);
        assertGt(ts, 0);
        assertTrue(active);
    }

    function test_nonAttesterCannotRegister() public {
        vm.prank(randomAddr);
        vm.expectRevert(HardTrustRegistry.NotAttester.selector);
        registry.registerDevice(serialHash, deviceAddr);
    }

    function test_unregisteredSerialReturnsZero() public view {
        bytes32 unknownHash = keccak256("UNKNOWN");
        (address dAddr, address att, uint256 ts, bool active) = registry.getDevice(unknownHash);
        assertEq(dAddr, address(0));
        assertEq(att, address(0));
        assertEq(ts, 0);
        assertFalse(active);
    }

    function test_isAttester() public view {
        assertTrue(registry.isAttester(attesterAddr));
        assertFalse(registry.isAttester(randomAddr));
    }

    function test_registerDevice_emitsEvent() public {
        vm.prank(attesterAddr);
        vm.expectEmit(true, true, true, true);
        emit HardTrustRegistry.DeviceRegistered(serialHash, deviceAddr, attesterAddr);
        registry.registerDevice(serialHash, deviceAddr);
    }

    function test_duplicateRegistration_reverts() public {
        vm.prank(attesterAddr);
        registry.registerDevice(serialHash, deviceAddr);

        address otherDevice = address(0xBEEF);
        vm.prank(attesterAddr);
        vm.expectRevert(abi.encodeWithSelector(HardTrustRegistry.DeviceAlreadyRegistered.selector, serialHash));
        registry.registerDevice(serialHash, otherDevice);
    }

    function test_duplicateRegistration_sameAddress_reverts() public {
        vm.prank(attesterAddr);
        registry.registerDevice(serialHash, deviceAddr);

        vm.prank(attesterAddr);
        vm.expectRevert(abi.encodeWithSelector(HardTrustRegistry.DeviceAlreadyRegistered.selector, serialHash));
        registry.registerDevice(serialHash, deviceAddr);
    }

    function test_duplicateRegistration_preservesOriginal() public {
        vm.prank(attesterAddr);
        registry.registerDevice(serialHash, deviceAddr);

        (address origAddr, address origAtt, uint256 origTs, bool origActive) = registry.getDevice(serialHash);

        address otherDevice = address(0xBEEF);
        vm.prank(attesterAddr);
        vm.expectRevert(abi.encodeWithSelector(HardTrustRegistry.DeviceAlreadyRegistered.selector, serialHash));
        registry.registerDevice(serialHash, otherDevice);

        (address dAddr, address att, uint256 ts, bool active) = registry.getDevice(serialHash);
        assertEq(dAddr, origAddr);
        assertEq(att, origAtt);
        assertEq(ts, origTs);
        assertEq(active, origActive);
    }

    function test_differentSerials_bothSucceed() public {
        bytes32 serial1 = keccak256("SERIAL-A");
        bytes32 serial2 = keccak256("SERIAL-B");
        address device1 = address(0xAA);
        address device2 = address(0xBB);

        vm.prank(attesterAddr);
        registry.registerDevice(serial1, device1);

        vm.prank(attesterAddr);
        registry.registerDevice(serial2, device2);

        (address d1,,, bool a1) = registry.getDevice(serial1);
        (address d2,,, bool a2) = registry.getDevice(serial2);
        assertEq(d1, device1);
        assertEq(d2, device2);
        assertTrue(a1);
        assertTrue(a2);
    }
}
