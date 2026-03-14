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
}
